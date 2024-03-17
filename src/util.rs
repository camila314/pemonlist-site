use std::fmt::Debug;

pub trait PrintErr {
    fn print_err(self) -> Self;
}

impl<T, E: Debug> PrintErr for Result<T, E> {
    fn print_err(self) -> Self {
        match &self {
            Ok(_) => (),
            Err(e) => println!("Error: {:?}", e)
        }

        self
    }
}