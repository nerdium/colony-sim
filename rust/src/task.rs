use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, rc::Rc};
use godot::prelude::*;

pub trait Task {
    fn next(&mut self, prev: Option<ActionResult>) -> Option<Box<dyn Action>>;
}

pub trait Action {
    fn start(self: Box<Self>, ctx: &mut ActionContext, result: ActionCallback);
}

pub type ActionData = Box<dyn Any>;
pub type ActionResult = Result<ActionData, String>;

pub struct ActionContext<'a> {
    pub host: Gd<Node>,
    pub caps: &'a mut Capabilities
}

#[derive(Clone)]
pub struct ActionCallback {
    inbox: Rc<RefCell<Option<ActionResult>>>
}

impl ActionCallback {
    pub fn new() -> Self {
        Self { inbox: Rc::new(RefCell::new(None)) }
    }

    pub fn success(self) {
        let result: ActionResult = Ok(Box::new(()));
        *self.inbox.borrow_mut() = Some(result);
    }

    pub fn success_with<T: Any>(self, data: T) {
        let result: ActionResult = Ok(Box::new(data));
        *self.inbox.borrow_mut() = Some(result);
    }

    pub fn failure(self, reason: impl Into<String>) {
        let result: ActionResult = Err(reason.into());
        *self.inbox.borrow_mut() = Some(result);
    }

    pub fn take(&mut self) -> Option<ActionResult> {
        self.inbox.borrow_mut().take()
    }
}

pub trait Capability {

}

#[derive(Default)]
pub struct Capabilities {
    map: HashMap<TypeId, Box<dyn Any>>
}

impl Capabilities {
    pub fn insert<C: Any>(&mut self, cap: C) {
        self.map.insert(TypeId::of::<C>(), Box::new(cap));
    }

    pub fn get<C: Any>(&self) -> Option<&C> {
        self.map
            .get(&TypeId::of::<C>())
            .and_then(|b| b.downcast_ref::<C>())
    }

    pub fn get_mut<C: Any>(&mut self) -> Option<&mut C> {
        self.map
            .get_mut(&TypeId::of::<C>())
            .and_then(|b| b.downcast_mut::<C>())
    }
}

pub struct TaskRunner {
    task: Option<Box<dyn Task>>,
    callback: ActionCallback,
    capabilities: Capabilities
}

impl TaskRunner {
    pub fn new(capabilities: Capabilities) -> Self {
        Self {
            task: None,
            callback: ActionCallback::new(),
            capabilities
        }
    }

    // TODO: probably shouldn't take host param, start action in poll
    pub fn assign_task(&mut self, mut task: Box<dyn Task>, host: Gd<Node>) {
        if self.is_busy() {
            godot_warn!("task runner is busy; task not started");
            return;
        }

        if let Some(first_action) = task.next(None) {
            self.task = Some(task);
            self.start_action(first_action, host);
        }
    }

    pub fn poll(&mut self, host: Gd<Node>) {
        // check for active task
        if let Some(task) = &mut self.task {
            // check if action has completed
            if let Some(action_result) = self.callback.take() {
                
                // fetch the next action from the task
                let Some(next_action) = task.next(Some(action_result)) else {
                    
                    // if no next action, then task is complete
                    self.task = None;
                    return;
                };

                self.start_action(next_action, host);
            }
        }
    }

    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }

    pub fn start_action(&mut self, action: Box<dyn Action>, host: Gd<Node>) {
        // build context
        let mut context = ActionContext { host, caps: &mut self.capabilities };
        action.start(&mut context, self.callback.clone());
    }
}