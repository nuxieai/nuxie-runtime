use crate::mechanical_port::source::component_dirt::ComponentDirt;

pub trait Dirtyable {
    fn add_dirt(&mut self, value: ComponentDirt, recurse: bool);
}
