macro_rules! mcp_return {
    ($expr:expr) => {
        return Ok(CallToolResult::success(vec![Content::text($expr)]))
    };
}
