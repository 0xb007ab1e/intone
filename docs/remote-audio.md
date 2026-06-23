# Remote audio: hearing intone while developing remotely

When you develop intone over SSH/tmux (no speakers on the dev box, or you're away from
it), there are two ways to perceive its speech:

1. **Text mode (no audio, simplest).** `INTONE_SPEECH=text` prints each announcement to the
   terminal — readable over SSH, no daemon, no audio. Best for logic/CI work. (Use
   `INTONE_SPEECH=both` to print *and* speak.)
2. **Route the audio to your machine** over the tunnel/tailnet — this page.

## How intone produces audio

```
intone --(SSIP socket)--> speech-dispatcher --(output module: espeak-ng)--> PipeWire/Pulse --> speakers
```

intone never touches the audio device directly. The audio sink is chosen by
**speech-dispatcher**, which honours the standard **`PULSE_SERVER`** environment variable
(Parrot/Debian use PipeWire's Pulse-compatible server). So "route intone's audio" really means
"point speech-dispatcher's `PULSE_SERVER` at your remote machine."

> **Key property:** intone auto-spawns speech-dispatcher and the daemon *inherits intone's
> environment*. So if `PULSE_SERVER` is set when you start intone, the speech it spawns uses
> it — no intone flag needed.
>
> **Gotcha:** intone only spawns the daemon if one isn't already running. A **stale local
> speech-dispatcher** will keep playing on the dev box and ignore `PULSE_SERVER`. Kill it
> first so intone respawns it with the right env: `pkill -u "$(id -u)" speech-dispatcher`.
> (`scripts/remote-audio.sh` does this for you.)

## Recipe A — over an SSH reverse tunnel

You're SSH'd from your **station** (has speakers) into the **dev box** (runs intone).

1. **On your station**, expose its PipeWire/Pulse over TCP (cookie-authenticated, localhost):
   ```sh
   pactl load-module module-native-protocol-tcp port=4713 auth-anonymous=0   # PipeWire/Pulse
   ```
2. **Reverse-forward** that port into the dev box when you connect (or add to `~/.ssh/config`):
   ```sh
   ssh -R 24713:localhost:4713 devbox
   ```
3. **On the dev box**, point speech-dispatcher at the forwarded socket and run intone:
   ```sh
   export PULSE_SERVER=tcp:localhost:24713
   scripts/remote-audio.sh            # restarts the daemon with this env, then runs intone
   ```

Audio now plays on your station. (You still need GUI focus changes on the dev box's desktop
to hear element announcements — see the spike notes.)

## Recipe B — over the tailnet (MagicDNS)

Both machines are on the same Tailscale tailnet (WireGuard-encrypted mesh).

1. **On your station**, bind PipeWire/Pulse TCP to the **tailnet interface IP** (never
   `0.0.0.0`):
   ```sh
   pactl load-module module-native-protocol-tcp listen=100.x.y.z port=4713 auth-anonymous=0
   ```
2. **On the dev box**:
   ```sh
   export PULSE_SERVER=tcp:station:4713        # MagicDNS name of your station
   scripts/remote-audio.sh
   ```

### Security (don't skip)

- The tailnet link is encrypted, but PulseAudio's TCP auth is weak — keep the **cookie auth**
  (don't use `auth-anonymous=1`) and gate access with **Tailscale ACLs**. Bind to the tailnet
  IP, **never `0.0.0.0`** and **never a public port-forward / Tailscale Funnel** (per the
  tailnet-dev-access rules).
- Speech can contain on-screen text (potentially sensitive). Treat the audio stream as
  confidential — keep it on the encrypted tailnet, not plaintext LAN.

## When you're at the desktop

Just run normally (`cargo run -p intone-linux`) — speech plays on the local speakers; none of
this is needed.

## Future work

A native option (skip speech-dispatcher's Pulse path and stream audio ourselves) is out of
scope; `PULSE_SERVER` is the standard, lowest-friction route. Tracked in the issue for #8.
