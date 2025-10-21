use serde_json::json;

/// Default workflow definitions built into the application
pub struct DefaultWorkflows;

impl DefaultWorkflows {
    /// Get the Simple Echo Demo workflow as JSON string
    pub fn simple_echo() -> String {
        serde_json::to_string_pretty(&json!({
            "id": "cursor_agent_demo",
            "name": "Cursor Agent JSON Parser Demo",
            "tasks": [
                {
                    "id": "echo_complete",
                    "name": "Echo Complete",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["Demo complete!"]
                        }
                    }
                }
            ]
        }))
        .unwrap()
    }

    /// Get the Cursor Agent Demo workflow as JSON string
    pub fn cursor_demo() -> String {
        serde_json::to_string_pretty(&json!({
            "id": "cursor_agent_demo",
            "name": "Cursor Agent JSON Parser Demo",
            "tasks": [
                {
                    "id": "who_are_you",
                    "name": "Ask Cursor Agent Who Are You",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "cursor",
                            "args": ["agent", "-p", "who are you"]
                        }
                    }
                },
                {
                    "id": "get_json_example",
                    "name": "Get JSON Example from Cursor Agent",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "response_type": "json",
                            "command": "cursor-agent",
                            "args": [
                                "--output-format",
                                "json",
                                "-p",
                                "give me an example JSON object with user information including name, age, email, and address"
                            ]
                        }
                    }
                },
                {
                    "id": "echo_processing",
                    "name": "Echo Processing",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["Processing JSON..."]
                        }
                    }
                },
                {
                    "id": "echo_extracting",
                    "name": "Echo Extracting",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["Extracting values..."]
                        }
                    }
                },
                {
                    "id": "echo_complete",
                    "name": "Echo Complete",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["Demo complete!"]
                        }
                    }
                }
            ]
        })).unwrap()
    }
}
