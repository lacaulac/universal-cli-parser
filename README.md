# universal-cli-parser

A blazingly-fast™ command-line argument parser written in Rust. Able to parse command-lines similarly to what bash does.

The main interest however is to use information derived from the [DID U Misbehave](https://github.com/lacaulac/DID-U-Misbehave) dataset to interpret the command-line arguments from a behavioural standpoint, yielding behaviour trees. UCP exposes two API paths on HTTP port 6880 :

- `/parse` : Parses the arguments into a behaviour tree (output is human-readable but not designed for machine consumption)
- `/behaviours` : Parses the arguments into a behaviour tree in JSON format

The behaviour taxonomy is that of [DID U Misbehave](https://github.com/lacaulac/DID-U-Misbehave).

This project is licensed under GNU General Public License v3.0.

## Running the project

- `git clone https://github.com/lacaulac/universal-cli-parser.git`
- `cd universal-cli-parser`
- `cargo run`

## Examples

*Note: UCP is not mature yet and is missing important features, such as program-specific inherent behaviour handling.*

```python
POST http://localhost:6880/behaviours

{
    "program": "tar",
    "args": ["-x", "--file", "archive.tar", "-v"]
}
```

```json
[
    {
        "CLInherentBehaviour": "FILE_READ"
    },
    {
        "CLInherentBehaviour": "FILE_WRITE"
    },
    {
        "CLBehaviouredOption": [
            "x",
            [
                "FILE_READ",
                "FILE_WRITE"
            ],
            null
        ]
    },
    {
        "CLBehaviouredOption": [
            "file",
            [
                "FILE_READ",
                "FILE_WRITE"
            ],
            {
                "String": "archive.tar"
            }
        ]
    },
    {
        "CLBehaviouredOption": [
            "v",
            [
                "NEUTRAL"
            ],
            null
        ]
    }
]
```

```python
POST http://localhost:6880/behaviours

{
    "program": "curl",
    "args": ["-o", "test.html", "https://example.com/test.zip", "-k"]
}
```

```json
[
    {
        "CLInherentBehaviour": "NET_COMS"
    },
    {
        "CLBehaviouredOption": [
            "o",
            [
                "FILE_WRITE"
            ],
            {
                "String": "test.html"
            }
        ]
    },
    {
        "CLArgument": {
            "URL": "https://example.com/test.zip"
        }
    },
    {
        "CLBehaviouredOption": [
            "k",
            [
                "NEUTRAL"
            ],
            null
        ]
    }
]
```

## Limitations
Weird syntaxes such as GNU `tar`'s `--checkpoint-action` (*e.g.*, `--checkpoint-action=exec=/bin/sh`) option are not handled well, as they should typically be split multiple times. In the format of the exemple above, there is a specific feature that tries to split on possible separators (as specified in the program's config file, a space character or an equal character in the case of tar), thus allowing the matching of `--checkpoint-action=exec` as one single option. However, this is only good enough for a crude proof-of-concept, as attackers could make the option unrecognised (*e.g.,* trying to parse `["--checkpoint-action", "exec=/bin/sh"]` would return an error).

The only way to fix this issue would be to have some sort of program-specific script, which is part of our plans for UCP.