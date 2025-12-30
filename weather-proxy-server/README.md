# Weather Proxy Server

This server acts as a proxy between the Weather Alerts app and the OpenWeatherMap API, keeping your API key secure on the server.

## Setup

1. Copy your `.env` file here (or create one with `OPENWEATHERMAP_API_KEY=your-key`)
2. Run the server: `cargo run`
3. The server will start on `http://localhost:3000`

## Deployment

For production, you'll need to:
1. Deploy this server to a hosting service (Heroku, Railway, DigitalOcean, AWS, etc.)
2. Set the `OPENWEATHERMAP_API_KEY` environment variable on your hosting service
3. Update the app to point to your server URL instead of localhost

## API Endpoint

POST `/api/weather`

Request body:
```json
{
  "city": "New York",
  "country_code": "US"
}
```

Response:
```json
{
  "weather_data": { ... },
  "daily_weather_description": "clear sky",
  "city": "New York"
}
```

