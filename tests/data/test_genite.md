# Genite Configuration

```cola
llm plural llms:
    openai:
        api:
            type: "REST",
            key: "some_api_key",
            base_url: "some_base_url"
        ;

        model plural models:
            gpt-4.1:
                name: "gpt-4.1",
                max_input_tokens: 1047576,
                max_output_tokens: 32768,
                input_price: 2.0,
                output_price: 8.0,
                supports_vision: true,
                supports_function_calling: true
            ;
            gpt-4.1-mini:
                name: "gpt-4.1-mini",
                max_input_tokens: 1047576,
                max_output_tokens: 32768,
                input_price: 0.4,
                output_price: 1.6,
                supports_vision: true,
                supports_function_calling: true
            ;
            gpt-4.1-nano:
                name: "gpt-4.1-nano",
                max_input_tokens: 1047576,
                max_output_tokens: 32768,
                input_price: 0.1,
                output_price: 0.4,
                supports_vision: true,
                supports_function_calling: true
            ;
            gpt-4o:
                name: "gpt-4o",
                max_input_tokens: 1047576,
                max_output_tokens: 32768,
                input_price: 2.0,
                output_price: 8.0,
                supports_vision: true,
                supports_function_calling: true
            ;
            o1:
                name: "o1",
                max_input_tokens: 128000,
                input_price: 15,
                output_price: 60,
                supports_vision: true,
                supports_function_calling: true
            ;
        ;
    ;

    gemini:
        api:
            type: "REST",
            key: "some_api_key",
            base_url: "some_base_url"
        ;

        model plural models:
            gemini-2.5-flash-preview-04-17:
                name: "gemini-2.5-flash-preview-04-17",
                max_input_tokens: 2097152,
                max_output_tokens: 8192,
                input_price: 0,
                output_price: 0,
                supports_vision: true,
                supports_function_calling: true
            ;
            gemini-1.5-pro-latest:
                name: "gemini-1.5-pro-latest",
                max_input_tokens: 2097152,
                max_output_tokens: 8192,
                input_price: 0,
                output_price: 0,
                supports_vision: true,
                supports_function_calling: true
            ;
            gemini-2.0-flash-thinking-exp:
                name: "gemini-2.0-flash-thinking-exp",
                max_input_tokens: 32768,
                max_output_tokens: 8192,
                input_price: 0,
                output_price: 0,
                supports_vision: true,
                supports_function_calling: true
            ;
            gemini-exp-1206:
                name: "gemini-exp-1206",
                max_input_tokens: 32768,
                max_output_tokens: 8192,
                input_price: 0,
                output_price: 0,
                supports_vision: true,
                supports_function_calling: true
            ;
        ;
    ;

    claude:
        api:
            type: "REST",
            key: "some_api_key",
            base_url: "some_base_url"
        ;

        model plural models:
            claude-3-7-sonnet-20250219:
                name: "claude-3-7-sonnet-20250219",
                max_input_tokens: 200000,
                max_output_tokens: 8192,
                require_max_tokens: true,
                input_price: 3,
                output_price: 15,
                supports_vision: true,
                supports_function_calling: true
            ;
            claude-3-7-sonnet-20250219-thinking:
                name: "claude-3-7-sonnet-20250219-thinking",
                max_input_tokens: 200000,
                max_output_tokens: 24000,
                require_max_tokens: true,
                input_price: 3,
                output_price: 15,
                supports_vision: true,
                supports_function_calling: true
            ;
        ;
    ;

    ollama:
        api:
            type: "REST",
            base_url: "some_base_url"
        ;

        model plural models:
            deepseek-r1-7b:
                name: "deepseek-r1:7b",
                max_input_tokens: 131072,
                input_price: 0,
                output_price: 0,
                supports_vision: true,
                supports_function_calling: true
            ;

            deepseek-r1-14b:
                name: "deepseek-r1:14b",
                max_input_tokens: 131072,
                input_price: 0,
                output_price: 0,
                supports_vision: true,
                supports_function_calling: true
            ;

            deepseek-r1-32b:
                name: "deepseek-r1:32b",
                max_input_tokens: 131072,
                input_price: 0,
                output_price: 0,
                supports_vision: true,
                supports_function_calling: true
            ;
        ;
    ;
;
```
