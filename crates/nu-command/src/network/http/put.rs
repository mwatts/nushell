use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
};

use crate::network::http::client::{
    http_client, http_parse_url, request_add_authorization_header, request_add_custom_headers,
    request_handle_response, request_set_body, request_set_timeout,
};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "http put"
    }

    fn signature(&self) -> Signature {
        Signature::build("http put")
            .input_output_types(vec![(Type::Nothing, Type::Any)])
            .allow_variants_without_examples(true)
            .required("URL", SyntaxShape::String, "the URL to post to")
            .required("data", SyntaxShape::Any, "the contents of the post body")
            .named(
                "user",
                SyntaxShape::Any,
                "the username when authenticating",
                Some('u'),
            )
            .named(
                "password",
                SyntaxShape::Any,
                "the password when authenticating",
                Some('p'),
            )
            .named(
                "content-type",
                SyntaxShape::Any,
                "the MIME type of content to post",
                Some('t'),
            )
            .named(
                "content-length",
                SyntaxShape::Any,
                "the length of the content being posted",
                Some('l'),
            )
            .named(
                "max-time",
                SyntaxShape::Int,
                "timeout period in seconds",
                Some('m'),
            )
            .named(
                "headers",
                SyntaxShape::Any,
                "custom headers you want to add ",
                Some('H'),
            )
            .switch(
                "raw",
                "return values as a string instead of a table",
                Some('r'),
            )
            .switch(
                "insecure",
                "allow insecure server connections when using SSL",
                Some('k'),
            )
            .filter()
            .category(Category::Network)
    }

    fn usage(&self) -> &str {
        "Put a body to a URL."
    }

    fn extra_usage(&self) -> &str {
        "Performs HTTP PUT operation."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["network", "send", "push"]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        run_put(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Put content to example.com",
                example: "http put https://www.example.com 'body'",
                result: None,
            },
            Example {
                description: "Put content to example.com, with username and password",
                example: "http put -u myuser -p mypass https://www.example.com 'body'",
                result: None,
            },
            Example {
                description: "Put content to example.com, with custom header",
                example: "http put -H [my-header-key my-header-value] https://www.example.com",
                result: None,
            },
            Example {
                description: "Put content to example.com, with JSON body",
                example: "http put -t application/json https://www.example.com { field: value }",
                result: None,
            },
        ]
    }
}

struct Arguments {
    url: Value,
    headers: Option<Value>,
    data: Value,
    content_type: Option<String>,
    content_length: Option<String>,
    raw: bool,
    insecure: bool,
    user: Option<String>,
    password: Option<String>,
    timeout: Option<Value>,
}

fn run_put(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    _input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let args = Arguments {
        url: call.req(engine_state, stack, 0)?,
        headers: call.get_flag(engine_state, stack, "headers")?,
        data: call.req(engine_state, stack, 1)?,
        content_type: call.get_flag(engine_state, stack, "content-type")?,
        content_length: call.get_flag(engine_state, stack, "content-length")?,
        raw: call.has_flag("raw"),
        insecure: call.has_flag("insecure"),
        user: call.get_flag(engine_state, stack, "user")?,
        password: call.get_flag(engine_state, stack, "password")?,
        timeout: call.get_flag(engine_state, stack, "max-time")?,
    };

    helper(engine_state, stack, call, args)
}

// Helper function that actually goes to retrieve the resource from the url given
// The Option<String> return a possible file extension which can be used in AutoConvert commands
fn helper(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    args: Arguments,
) -> Result<PipelineData, ShellError> {
    let span = args.url.span()?;
    let (requested_url, url) = http_parse_url(call, span, args.url)?;

    let client = http_client(args.insecure);
    let mut request = client.put(url);

    request = request_set_body(args.content_type, args.content_length, args.data, request)?;
    request = request_set_timeout(args.timeout, request)?;
    request = request_add_authorization_header(args.user, args.password, request);
    request = request_add_custom_headers(args.headers, request)?;

    let response = request.send().and_then(|r| r.error_for_status());
    request_handle_response(
        engine_state,
        stack,
        span,
        &requested_url,
        args.raw,
        response,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}