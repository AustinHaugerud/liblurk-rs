use super::protocol_message::LurkMessageFrame;
use super::extractor::*;

use std::io::Read;
use std::borrow::Borrow;

use std::rc::Rc;

pub trait EventDistributor {
  fn distribute_event(&mut self, event : &LurkEvent);
}

#[derive(Clone)]
pub struct LurkEvent {
  pub frame : LurkMessageFrame
}

pub struct EventReceiver {
  handlers : Vec<fn(&LurkEvent)>,
  event_queue : Vec<LurkEvent>,
}

impl EventDistributor for EventReceiver {

  fn distribute_event(&mut self, event: &LurkEvent) {
    let event_op = self.event_queue.pop();

    if event_op.is_some() {

      let event = event_op.unwrap();

      for handler in self.handlers.iter() {
        handler(&event.clone());
      }
    }
  }
}

impl EventReceiver {
  pub fn new() -> EventReceiver {
    EventReceiver { handlers : vec![], event_queue : vec![] }
  }

  pub fn bind_handler(&mut self, handler : fn(&LurkEvent)) {
    self.handlers.push(handler);
  }

  pub fn push_event(&mut self, event : LurkEvent) {
    self.event_queue.push(event);
  }
}

#[cfg(test)]
mod tests {
  use ::protocol::protocol_message::LurkMessageFrame;
  use super::*;

  #[test]
  fn test_handler_bind() {
    let frame = LurkMessageFrame::new(0xFF, vec![]);
    let event = LurkEvent { frame };
    let mut receiver = EventReceiver { handlers : vec![], event_queue : vec![] };
    receiver.bind_handler(|event|{});
    receiver.distribute_event(&event);
  }

}
