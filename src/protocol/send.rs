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
        self.target.write_all(&mut data).map_err(|_| ())
    }

    pub fn write_message_uptr(
        &mut self,
        message: &Box<LurkMessageBlobify + Send>,
    ) -> Result<(), ()> {
        let mut data = message.produce_lurk_message_blob();
        self.target.write_all(&mut data).map_err(|_| ())
    }

    pub fn write_message_ref_dyn(&mut self, message : &LurkMessageBlobify) -> Result<(), ()> {
        let mut data = message.produce_lurk_message_blob();
        self.target.write_all(&mut data).map_err(|_| ())
    }
}
