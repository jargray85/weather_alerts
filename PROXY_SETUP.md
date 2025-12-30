# Weather Proxy Server Setup

This app now uses a proxy server to keep your API key secure. The API key stays on your server and is never exposed to end users.

## Architecture

```
Weather Alerts App → Your Proxy Server → OpenWeatherMap API
                    (API key here)      (never exposed)
```

## Setup Steps

### 1. Start the Proxy Server

```bash
cd weather-proxy-server
cargo run
```

The server will start on `http://localhost:3000`

### 2. Test the App Locally

The app is configured to use `http://localhost:3000` by default. Just run:

```bash
cargo run
```

### 3. For Production/Distribution

**Option A: Bundle the Server URL**

Update the app to use your production server URL. You can:
- Set `WEATHER_PROXY_URL` environment variable when building
- Or hardcode the production URL in the code

**Option B: Deploy the Server**

Deploy `weather-proxy-server` to a hosting service:
- **Heroku**: Free tier available
- **Railway**: Easy deployment
- **DigitalOcean**: $5/month droplet
- **AWS/GCP**: More complex but scalable

Set the `OPENWEATHERMAP_API_KEY` environment variable on your hosting service.

### 4. Update App for Production

Before building the app for distribution, set the server URL:

```bash
export WEATHER_PROXY_URL="https://your-server.com"
cargo tauri build
```

Or hardcode it in `src/main.rs`:
```rust
let proxy_url = "https://your-server.com".to_string();
```

## Security Benefits

✅ API key never leaves your server  
✅ API key not in app bundle  
✅ Users cannot extract your API key  
✅ You can monitor/rate limit requests  
✅ You can rotate API keys without updating the app  

## Server Requirements

- Rust runtime
- Port 3000 (or configure a different port)
- `OPENWEATHERMAP_API_KEY` environment variable
- Internet access to call OpenWeatherMap API

