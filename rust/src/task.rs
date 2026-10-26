use std::{any::Any, cell::RefCell, rc::Rc};
use godot::prelude::*;

pub type ActionData = Box<dyn Any>;
pub type ActionResult = Result<ActionData, String>;
pub type ActionInbox = Rc<RefCell<Option<ActionResult>>>;

pub trait ActionResultExt {
    fn data<T: 'static>(&self) -> Option<&T>;
    fn into_data<T: 'static>(self) -> Option<T>;
}

impl ActionResultExt for ActionResult {
    fn data<T: 'static>(&self) -> Option<&T> {
        self.as_ref().ok()?.downcast_ref::<T>()
    }

    fn into_data<T: 'static>(self) -> Option<T> {
        self.ok()?.downcast::<T>().ok().map(|b| *b)
    }
}

pub trait Action {
    fn start(self: Box<Self>, host: Gd<Node>, done: Completion);
}

pub trait Task {
    fn next(&mut self, prev: Option<ActionResult>) -> Option<Box<dyn Action>>;
}

pub struct Completion {
    pub runner: Gd<TaskRunner>,
    pub inbox: ActionInbox,
    sent: bool
}

impl Completion {
    pub fn new(runner: Gd<TaskRunner>, inbox: ActionInbox) -> Self {
        Self { runner, inbox, sent: false }
    }

    pub fn finish(&mut self, result: ActionResult) {
        self.sent = true;
        if !self.runner.is_instance_valid() {
            return; // runner was freed while action was in flight
        }

        *self.inbox.borrow_mut() = Some(result);
        self.runner.call_deferred("advance_task", &[]);
    }

    pub fn success(mut self) {
        self.finish(Ok(Box::new(())));
    }

    pub fn success_with<T: Any>(mut self, data: T) {
        self.finish(Ok(Box::new(data)));
    }

    pub fn failed(mut self, reason: impl Into<String>) {
        self.finish(Err(reason.into()));
    }
}

impl Drop for Completion {
    fn drop(&mut self) {
        if !self.sent {
            self.finish(Err("action dropped before completion was sent".into()));
        }
    }
}

#[derive(GodotClass)]
#[class(base = Node, init)]
pub struct TaskRunner {
    base: Base<Node>,

    #[export] host: Option<Gd<Node>>,

    task: Option<Box<dyn Task>>,
    inbox: ActionInbox
}

#[godot_api]
impl TaskRunner {
    #[signal]
    fn task_finished();

    pub fn start_task(&mut self, task: Box<dyn Task>) -> bool {
        if self.is_busy() {
            godot_warn!("task runner is busy; task not started");
            return false;
        }

        self.task = Some(task);
        self.base_mut().call_deferred("advance_task", &[]);
        true
    }

    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }

    #[func]
    fn advance_task(&mut self) {
        let Some(task) = self.task.as_mut() else { return } ;

        let Some(next_action) = task.next(self.inbox.borrow_mut().take()) else {
            self.finish_task();
            return;
        };

        let Some(runner_host) = self.resolve_host() else {
            godot_error!("task runner has no host");
            self.finish_task();
            return;
        };

        let completion_handle = Completion::new(self.to_gd(), self.inbox.clone());

        next_action.start(runner_host, completion_handle);
    }

    fn finish_task(&mut self) {
        self.task = None;
        self.base_mut().call_deferred("emit_signal", &["task_finished".to_variant()]);
    }

    fn resolve_host(&self) -> Option<Gd<Node>> {
        self.host.clone().or_else(|| self.base().get_parent())
    }
}