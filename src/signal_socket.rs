use actix::prelude::{Actor, ActorContext, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use futures::executor::block_on;

use super::{Error, ExitMessage, JoinMessage, Signal, SignalMessage, SignalRouter};

pub struct SignalSocket {
    user_name: String,
    signal_router: Addr<SignalRouter>,
}

impl SignalSocket {
    pub fn new<T: ToString>(user_name: T, signal_router: &Addr<SignalRouter>) -> Self {
        SignalSocket {
            user_name: user_name.to_string(),
            signal_router: signal_router.clone(),
        }
    }

    async fn handle_signal_message(
        &self,
        signal_message: Signal,
        context: &mut ws::WebsocketContext<Self>,
    ) {
        let signal_routing_result = self
            .signal_router
            .send(SignalMessage::from(signal_message))
            .await
            .unwrap_or_else(into_service_releated_error);
        if let Err(err) = signal_routing_result {
            context.text(serde_json::to_string(&ErrorMessage::from(err)).unwrap())
        }
    }
}

fn into_service_releated_error<T>(mailbox_error: actix::MailboxError) -> Result<T, Error> {
    Err(match mailbox_error {
        actix::MailboxError::Closed => Error::ServiceUnavailable,
        actix::MailboxError::Timeout => Error::ServiceTimeout,
    })
}

impl Actor for SignalSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let joining_router_fut = self.signal_router.send(JoinMessage::new(
            self.user_name.clone(),
            context.address().recipient(),
        ));
        let future = block_on(joining_router_fut);
        if future.is_ok() {
            let res = future.unwrap();
            match  res {
                Ok(_) => {
                    context.text(serde_json::to_string(&Signal::assign(self.user_name.clone())).unwrap());
                    println!("Signal Socket Opened")
                },
                Err(_) => {
                    println!("Signal Socket Closed");

                    context.text(serde_json::to_string(&ErrorMessage::from(Error::UserAlreadyJoined(self.user_name.clone()))).unwrap());
                    context.stop();
                },
            }
          
        } else {
           context.stop();
        }
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        let exiting_router_fut = self
            .signal_router
            .send(ExitMessage::from(self.user_name.clone()));

        if block_on(exiting_router_fut).is_ok() {
            println!("Signal Socket Closed")
        } else {
            eprintln!("couldn't exit from router. user name: {}", self.user_name)
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SignalSocket {
    fn handle(
        &mut self,
        message: Result<ws::Message, ws::ProtocolError>,
        context: &mut Self::Context,
    ) {
        match message {
            Ok(ws::Message::Close(_)) => {
                println!("close request received. closing.");
                context.stop();
            }
            Ok(ws::Message::Text(text_message)) => {
                println!("{}", text_message.to_string());
                match serde_json::from_str(&text_message) {
                    Ok(signal) => {
                        block_on(self.handle_signal_message(signal, context))
                    },
                    Err(err) => {
                        eprintln!("could not parse the message. {}", err.to_string());
                        context.text(format!("could not parse the message. {}", err.to_string()))
                    },
                }
            } 
            Ok(_) => {
                println!("some message received.");
            }
            Err(error) => eprintln!("error occurred during receive message: {}", error),
        }
    }
}

impl Handler<Signal> for SignalSocket {
    type Result = Result<(), Error>;

    fn handle(&mut self, message: Signal, context: &mut Self::Context) -> Self::Result {
        context.text(serde_json::to_string(&message)?);
        Ok(())
    }
}

#[derive(serde::Serialize)]
struct ErrorMessage {
    r#type: &'static str,
    message: String,
}

impl From<Error> for ErrorMessage {
    fn from(message_send_error: Error) -> Self {
        match message_send_error {
            Error::ParseError(parse_error) => ErrorMessage {
                r#type: "parse error",
                message: format!("{}", parse_error),
            },
            Error::ConnectionClosed => ErrorMessage {
                r#type: "connection closed",
                message: "target user's connection is closed".to_owned(),
            },
            Error::ConnectionTimeout => ErrorMessage {
                r#type: "timeout",
                message: "timeout occurres during send message to target user".to_owned(),
            },
            Error::TargetNotFound(target_user_name) => ErrorMessage {
                r#type: "target user not found",
                message: format!("user {} is not in connection", target_user_name),
            },
            Error::ServiceUnavailable => ErrorMessage {
                r#type: "service unavailable",
                message: "service is unavailable, please contact to service provider".to_owned(),
            },
            Error::ServiceTimeout => ErrorMessage {
                r#type: "service timeout",
                message: "service is busy. try after".to_owned(),
            },
            Error::UserAlreadyJoined(user) => ErrorMessage {
                r#type: "user already joined",
                message: format!("User '{}' already joined", user),
            },

        }
    }
}

