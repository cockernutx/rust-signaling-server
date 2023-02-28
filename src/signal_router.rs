use super::signal::Signal;
use actix::fut::wrap_future;
use actix::prelude::{Actor, Context, Handler, Message, Recipient, ResponseActFuture};
use futures::TryFutureExt;
use std::collections::HashMap;
use std::future::Future;


use super::Error;

#[derive(Default)]
pub struct SignalRouter {
    sockets: HashMap<String, Recipient<Signal>>,
}

impl Actor for SignalRouter {
    type Context = Context<Self>;
}

impl SignalRouter {
    fn target(&self, target_name: &str) -> Option<&Recipient<Signal>> {
        self.sockets.get(target_name)
    }

    fn wrap_future<F>(future: F) -> ResponseActFuture<Self, Result<(), Error>>
    where
        F: Future<Output = Result<(), Error>> + 'static,
    {
        Box::pin(wrap_future(future))
    }
}

impl Handler<SignalMessage> for SignalRouter {
    type Result = ResponseActFuture<Self, Result<(), Error>>;

    fn handle(&mut self, message: SignalMessage, _: &mut Self::Context) -> Self::Result {
        match &message.0 {
            Signal::Answer(signal) | Signal::Offer(signal) => {
                match self.target(&signal.target) {
                    Some(target_socket) => {
                        let message_transfer_future = target_socket
                        .send(message.0)
                        .unwrap_or_else(into_target_related_error);
                    return Self::wrap_future(message_transfer_future)
                    },
                    None =>{
                        return  Self::wrap_future(futures::future::err(Error::TargetNotFound(
                            signal.target.clone(),
                        )))
                    },
                } 
            }
            Signal::NewIceCandidate(ice_candidate) => {
                match self.target(&ice_candidate.target) {
                    Some(target_socket) => {
                        let message_transfer_future = target_socket
                        .send(message.0)
                        .unwrap_or_else(into_target_related_error);
                        return Self::wrap_future(message_transfer_future);
                    },
                    None => {
                        return Self::wrap_future(futures::future::err(Error::TargetNotFound(
                            ice_candidate.target.clone(),
                        )))
                    },
                } 
            }
            _ => Self::wrap_future(futures::future::ok(())), //do nothing
        }
    }
}

fn into_target_related_error<T>(mailbox_error: actix::MailboxError) -> Result<T, Error> {
    Err(match mailbox_error {
        actix::MailboxError::Closed => Error::ConnectionClosed,
        actix::MailboxError::Timeout => Error::ConnectionTimeout,
    })
}

impl Handler<JoinMessage> for SignalRouter {
    type Result = <JoinMessage as Message>::Result;

    fn handle(&mut self, message: JoinMessage, _: &mut Self::Context) -> Self::Result {
        for a in &self.sockets {
            println!("{}", a.0)
        }
        
        if self.sockets.contains_key(&message.user_name) {
            println!("The user {} already joined.", message.user_name);
            return Err(());
        }
        
        self.sockets
            .insert(message.user_name, message.signal_recipient);
        Ok(())
    }
}

impl Handler<ExitMessage> for SignalRouter {
    type Result = <JoinMessage as Message>::Result;

    fn handle(&mut self, message: ExitMessage, _: &mut Self::Context) -> Self::Result {
        self.sockets.remove(&message.0);
        Ok(())
    }
}

pub struct SignalMessage(Signal);

impl Message for SignalMessage {
    type Result = Result<(), Error>;
}

impl From<Signal> for SignalMessage {
    fn from(signal: Signal) -> Self {
        SignalMessage(signal)
    }
}

pub struct JoinMessage {
    user_name: String,
    signal_recipient: Recipient<Signal>,
}

impl JoinMessage {
    pub fn new(user_name: String, signal_recipient: Recipient<Signal>) -> Self {
        JoinMessage {
            user_name,
            signal_recipient,
        }
    }
}

impl Message for JoinMessage {
    type Result = Result<(), ()>;
}

pub struct ExitMessage(String);

impl Message for ExitMessage {
    type Result = Result<(), ()>;
}

impl From<String> for ExitMessage {
    fn from(name: String) -> Self {
        ExitMessage(name)
    }
}

#[cfg(test)]
mod test {
    use super::{JoinMessage, Signal, SignalMessage, SignalRouter};
    use actix::prelude::{Actor, Addr, Context, Handler, Message};
    use std::sync::{Arc, Mutex};

    #[actix_rt::test]
    async fn test_messaging() -> std::io::Result<()> {
        //given
        let testing_env = RouteTestingEnvironment::new().await;
        let signal_text = format!(
            "{{\"type\":\"offer\",\"name\":\"{}\",\"target\":\"{}\",\"sdp\":\"dummy sdp\"}}",
            RouteTestingEnvironment::caller_name(),
            RouteTestingEnvironment::callee_name()
        );
        let offer_signal: Signal = serde_json::from_str(&signal_text).unwrap();

        //when
        let signal_result = testing_env
            .router_addr
            .send(SignalMessage::from(offer_signal.clone()))
            .await;

        assert!(signal_result.is_ok());
        assert!(signal_result.unwrap().is_ok());

        Ok(())
    }

    #[actix_rt::test]
    async fn test_routing() -> std::io::Result<()> {
        //given
        let testing_env = RouteTestingEnvironment::new().await;
        let signal_text = format!(
            "{{\"type\":\"offer\",\"name\":\"{}\",\"target\":\"{}\",\"sdp\":\"dummy sdp\"}}",
            RouteTestingEnvironment::caller_name(),
            RouteTestingEnvironment::callee_name()
        );
        let offer_signal: Signal = serde_json::from_str(&signal_text).unwrap();

        //when
        testing_env
            .router_addr
            .send(SignalMessage::from(offer_signal.clone()))
            .await
            .unwrap()
            .unwrap();

        //then
        let resolved_signal_ref: &mut Option<Signal> =
            &mut testing_env.last_received_message.lock().unwrap();
        assert!(resolved_signal_ref.is_some());
        assert_eq!(&offer_signal, resolved_signal_ref.as_ref().unwrap());

        Ok(())
    }

    struct MockSignalHandler {
        last_received_message: Arc<Mutex<Option<Signal>>>,
    }

    impl MockSignalHandler {
        fn new(message_placeholder: Arc<Mutex<Option<Signal>>>) -> Self {
            MockSignalHandler {
                last_received_message: message_placeholder,
            }
        }
    }

    impl Actor for MockSignalHandler {
        type Context = Context<Self>;
    }

    impl Handler<Signal> for MockSignalHandler {
        type Result = <Signal as Message>::Result;

        fn handle(&mut self, message: Signal, _: &mut Self::Context) -> Self::Result {
            self.last_received_message.lock().unwrap().replace(message);
            Ok(())
        }
    }

    struct RouteTestingEnvironment {
        last_received_message: Arc<Mutex<Option<Signal>>>,
        router_addr: Addr<SignalRouter>,
    }

    impl RouteTestingEnvironment {
        async fn new() -> Self {
            let router_addr = SignalRouter::default().start();
            let message_placeholder: Arc<Mutex<Option<Signal>>> = Default::default();
            let caller_addr = MockSignalHandler::new(message_placeholder.clone()).start();
            let callee_addr = MockSignalHandler::new(message_placeholder.clone()).start();

            router_addr
                .send(JoinMessage::new(
                    Self::caller_name().to_owned(),
                    caller_addr.clone().recipient(),
                ))
                .await
                .expect("failed to join")
                .expect("failed to join");
            router_addr
                .send(JoinMessage::new(
                    Self::callee_name().to_owned(),
                    callee_addr.clone().recipient(),
                ))
                .await
                .expect("failed to join")
                .expect("failed to join");

            RouteTestingEnvironment {
                last_received_message: message_placeholder,
                router_addr,
            }
        }

        const fn caller_name() -> &'static str {
            "caller"
        }

        const fn callee_name() -> &'static str {
            "callee"
        }
    }
}