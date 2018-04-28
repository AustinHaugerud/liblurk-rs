use protocol::protocol_message::LurkMessageBlobify;
use std::io::Write;

pub struct LurkSendChannel<'a, T>
where
    T: 'a + Write,
{
    target: &'a mut T,
}

impl<'a, T> LurkSendChannel<'a, T>
where
    T: Write,
{
    pub fn new(target: &'a mut T) -> LurkSendChannel<'a, T> {
        LurkSendChannel { target }
    }

    pub fn write_message<F>(&mut self, message: F) -> Result<(), ()>
    where
        F: LurkMessageBlobify,
    {
        self.write_message_ref(&message)
    }

    pub fn write_message_ref<F>(&mut self, message: &F) -> Result<(), ()>
    where
        F: LurkMessageBlobify,
    {
        let mut data = message.produce_lurk_message_blob();

        if self.target.write_all(&mut data).is_err() {
            return Err(());
        }
        Ok(())
    }
}
