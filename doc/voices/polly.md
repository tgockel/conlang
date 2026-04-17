# AWS Polly

AWS Polly is a cloud-based text-to-speech service that produces natural-sounding neural speech.
It accepts IPA directly through SSML, so there is no phoneme conversion step -- what you write in your phonetic
inventory is what Polly tries to pronounce.

> **NOTE**
>
> If you just want the configuration fields, skip to [Configuration](#configuration).

## Setup

Enable the feature flag when building:

```sh
cargo build --features voice-polly
```

Polly requires **AWS credentials**.
The simplest method is to set environment variables:

```sh
export AWS_ACCESS_KEY_ID=your-key
export AWS_SECRET_ACCESS_KEY=your-secret
export AWS_REGION=us-east-1
```

Alternatively, configure `~/.aws/credentials` using the AWS CLI.
The region determines which voices are available -- most voices exist in `us-east-1`.

You can verify that credentials are working by running `conlang config voice scan`, which queries the Polly API
and lists available voices for your configured region.

## Configuration

Each Polly voice entry requires `"driver": "polly"` and accepts the following optional fields:

| Field      | Type   | Default     | Description                                           |
|:-----------|:-------|:------------|:------------------------------------------------------|
| `driver`   | string | *(required)* | Must be `"polly"`                                    |
| `voice_id` | string | `"Joanna"`  | Polly voice identifier (e.g., `"Joanna"`, `"Matthew"`) |
| `engine`   | string | `"neural"`  | `"neural"` for neural TTS or `"standard"` for classic |

A minimal entry:

```json
{
  "driver": "polly"
}
```

This uses the Joanna voice with the neural engine.

A multi-voice setup:

```json
{
  "polly-joanna": {
    "driver": "polly",
    "voice_id": "Joanna",
    "engine": "neural"
  },
  "polly-matthew": {
    "driver": "polly",
    "voice_id": "Matthew",
    "engine": "standard"
  }
}
```

The `"neural"` engine produces higher-quality speech but is not available for every voice ID.
The `"standard"` engine is available for all voices and is less expensive per character.

## How IPA Reaches Polly

When you speak through a Polly voice, the toolkit wraps your IPA string in an SSML `<phoneme>` tag:

```xml
<phoneme alphabet="ipa" ph="ˈpa.ta">.</phoneme>
```

Polly interprets the IPA directly, so there is no lossy conversion step.
This gives Polly higher IPA fidelity than [eSpeak-ng](espeak.md) for most sounds, though Polly's phoneme
support is limited to the sounds that exist in its voice's base language.

## Discovering Voices

Run `conlang config voice scan` to query the Polly `DescribeVoices` API:

```sh
conlang config voice scan
```

This lists every voice available in your configured AWS region, along with the voice ID, language, and
supported engines.
Use the voice IDs from this list as the `"voice_id"` value in your configuration.
