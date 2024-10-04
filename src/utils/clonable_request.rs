use reqwest::Request;
use std::ops::{Deref, DerefMut};

pub struct ClonableRequest(Request);

impl ClonableRequest {
    pub fn new(request: Request) -> Self {
        ClonableRequest(request)
    }

    pub fn into_inner(self) -> Request {
        self.0
    }
}

impl Deref for ClonableRequest {
    type Target = Request;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClonableRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Clone for ClonableRequest {
    fn clone(&self) -> Self {
        ClonableRequest(self.0.try_clone().unwrap())
    }
}