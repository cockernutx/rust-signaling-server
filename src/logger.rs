
macro_rules! logln {

    (error => $($arg:tt)+) => (

        {let now = chrono::prelude::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        println!("ERROR[{}, \x1b[41m\"{}\"\x1b[0m]", now, format!($($arg)+))}
        
    );
    (warning => $($arg:tt)+) => (
        {let now = chrono::prelude::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        println!("Warning[{},  \x1b[43m\"{}\"\x1b[0m]", now, format!($($arg)+))}
        
    );
    (info => $($arg:tt)+) => (
        {
            let now = chrono::prelude::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        println!("Info[{}, \x1b[32m\x1b[44m\"{}\"\x1b[0m]", now, format!($($arg)+))
    }
        
    );
    
}



pub(crate) use logln;
