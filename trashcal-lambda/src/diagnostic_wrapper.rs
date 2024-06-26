// https://github.com/awslabs/aws-lambda-rust-runtime/pull/897#issuecomment-2191309171
use std::{borrow::Cow, fmt::Display};

use lambda_runtime::Diagnostic;

pub struct DiagnosticWrapper(Diagnostic<'static>);

impl<'a> From<DiagnosticWrapper> for Diagnostic<'a> {
    fn from(value: DiagnosticWrapper) -> Self {
        value.0
    }
}

impl std::fmt::Debug for DiagnosticWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl DiagnosticWrapper {
    pub fn new<E>(err: E) -> Self
    where
        E: Display,
    {
        DiagnosticWrapper(Diagnostic {
            error_type: Cow::Borrowed(std::any::type_name::<E>()),
            error_message: Cow::Owned(err.to_string()),
        })
    }
}
