# ChatGPT in terminal

## Install
Use `cargo` install `chatgpt-cmd`

## Configuration
The configuration, `config.json`, and history, `history.json`, in under `~/.chatgpt`

### Config.json
 - url: URL to openai chat. Default is "https://api.openai.com/v1/chat/completions"
 - model: Chatgpt model. Default is "gpt-3.5-turbo"
 - key: Your API-key.

### History.json
 - hint: hint for system
 - history: history messages.

## Usage
`chatgpt-cmd -h` shows usage

```
Usage: chatgpt-cmd [OPTIONS] [prompt]

Arguments:
  [prompt]

Options:
  -c, --clean        Clean history
  -H, --hint <hint>  Hint ChatGPT
  -h, --help         Print help
```

For example:
`chatgpt-cmd "how to write hello world in Rust"`
