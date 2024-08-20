# winapi_to_json

An automatic collector of function metadata provided by the official Microsoft repo [microsoft/win32metadata](https://github.com/microsoft/win32metadata).

The project leverages the `windows_metadata` rust library provided by Microsoft, in order to parse the latest Windows Metadata file obtained from [https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.Win32.winmd](https://github.com/microsoft/windows-rs/raw/master/crates/libs/bindgen/default/Windows.Win32.winmd). After downloading and parsing the file, it will process the data to JSON format with the following structure:

```
[
  {
    "module_name": "example.dll",
    "functions": [
      {
        "function_name": "CreateExampleHandle",
        "ret_type": "HANDLE",
        "params": [
          "*Void examplePtr",
          "U32 exampleCommand",
          ...
        ]
      }
    ]
  },
  {
    "module_name": "...",
    "functions": [...]
  },
  ...
]
```


## How to obtain the json file?

You can obtain the output file in two ways:

- By downloading it from the "Releases" tab.
- By cloning and running the project locally.
