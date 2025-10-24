use serde_json::json;

pub struct DefaultWorkflows;

impl DefaultWorkflows {
    pub fn simple_echo() -> String {
        serde_json::to_string_pretty(&json!({
            "id": "cursor_agent_demo",
            "name": "Echo CLI Demo",
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

    pub fn cursor_demo() -> String {
        serde_json::to_string_pretty(&json!({
            "id": "cursor_agent_demo",
            "name": "5 Step CLI Demo",
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

    pub fn cursor_agent_simple() -> String {
        serde_json::to_string_pretty(&json!({
            "id": "cursor_agent_simple",
            "name": "Simple Cursor Agent Demo",
            "tasks": [
                {
                    "id": "ask_who_are_you",
                    "name": "Ask Cursor Agent Who Are You",
                    "function": {
                        "name": "cursor_agent",
                        "response_type": "json",
                        "input": {
                            "prompt": "who are you"
                        }
                    }
                }
            ]
        }))
        .unwrap()
    }

    pub fn user_input_demo() -> String {
        serde_json::to_string_pretty(&json!({
            "id": "user_input_demo",
            "name": "User Input Pause Demo",
            "description": "Demonstrates the user input pause functionality with multiple pause points",
            "tasks": [
                {
                    "id": "welcome_message",
                    "name": "Welcome Message",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["🚀 Starting User Input Demo Workflow"]
                        }
                    }
                },
                {
                    "id": "pause_for_name",
                    "name": "Pause for User Name",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["⏸️  Workflow paused - waiting for user input..."]
                        },
                        "pause_for_input": {
                            "prompt": "Please enter your name:",
                            "variable": "user_name"
                        }
                    }
                },
                {
                    "id": "greet_user",
                    "name": "Greet User",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["Hello ${user_name}! Nice to meet you."]
                        }
                    }
                },
                {
                    "id": "pause_for_choice",
                    "name": "Pause for User Choice",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["⏸️  Workflow paused again - waiting for your choice..."]
                        },
                        "pause_for_input": {
                            "prompt": "What would you like to do? (1) Continue, (2) Exit:",
                            "variable": "user_choice"
                        }
                    }
                },
                {
                    "id": "process_choice",
                    "name": "Process User Choice",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["You chose: ${user_choice}"]
                        }
                    }
                },
                {
                    "id": "final_message",
                    "name": "Final Message",
                    "function": {
                        "name": "cli_command",
                        "input": {
                            "command": "echo",
                            "args": ["✅ Demo complete! This workflow demonstrated user input pauses."]
                        }
                    }
                }
            ]
        }))
        .unwrap()
    }
}
