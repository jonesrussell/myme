use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_weather::{TemperatureUnit, WeatherCache, WeatherData, WeatherProvider};

use crate::bridge;
use crate::services::{request_weather_fetch, WeatherServiceMessage};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(bool, has_data)]
        #[qproperty(bool, is_stale)]
        #[qproperty(QString, error_message)]
        // Current weather properties
        #[qproperty(f64, temperature)]
        #[qproperty(f64, feels_like)]
        #[qproperty(i32, humidity)]
        #[qproperty(f64, wind_speed)]
        #[qproperty(QString, condition)]
        #[qproperty(QString, condition_icon)]
        #[qproperty(QString, location_name)]
        // Today's forecast
        #[qproperty(f64, today_high)]
        #[qproperty(f64, today_low)]
        #[qproperty(i32, precipitation_chance)]
        #[qproperty(QString, sunrise)]
        #[qproperty(QString, sunset)]
        type WeatherModel = super::WeatherModelRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut WeatherModel>);

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut WeatherModel>);

        #[qinvokable]
        fn set_temperature_unit(self: Pin<&mut WeatherModel>, unit: &QString);

        #[qinvokable]
        fn forecast_count(self: &WeatherModel) -> i32;

        #[qinvokable]
        fn get_forecast_date(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_day(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_high(self: &WeatherModel, index: i32) -> f64;

        #[qinvokable]
        fn get_forecast_low(self: &WeatherModel, index: i32) -> f64;

        #[qinvokable]
        fn get_forecast_condition(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_icon(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_precip(self: &WeatherModel, index: i32) -> i32;

        #[qinvokable]
        fn get_forecast_sunrise(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_sunset(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn hourly_count(self: &WeatherModel, day_index: i32) -> i32;

        #[qinvokable]
        fn get_hourly_time(self: &WeatherModel, day_index: i32, hour_index: i32) -> QString;

        #[qinvokable]
        fn get_hourly_temp(self: &WeatherModel, day_index: i32, hour_index: i32) -> f64;

        #[qinvokable]
        fn get_hourly_icon(self: &WeatherModel, day_index: i32, hour_index: i32) -> QString;

        #[qinvokable]
        fn get_hourly_precip(self: &WeatherModel, day_index: i32, hour_index: i32) -> i32;

        #[qsignal]
        fn weather_changed(self: Pin<&mut WeatherModel>);

        #[qsignal]
        fn error_occurred(self: Pin<&mut WeatherModel>);
    }
}

/// Operation state tracking
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum OpState {
    #[default]
    Idle,
    Fetching,
}

#[derive(Default)]
pub struct WeatherModelRust {
    loading: bool,
    has_data: bool,
    is_stale: bool,
    error_message: QString,
    // Current weather
    temperature: f64,
    feels_like: f64,
    humidity: i32,
    wind_speed: f64,
    condition: QString,
    condition_icon: QString,
    location_name: QString,
    // Today's forecast
    today_high: f64,
    today_low: f64,
    precipitation_chance: i32,
    sunrise: QString,
    sunset: QString,
    // Internal state
    weather_data: Option<WeatherData>,
    provider: Option<Arc<WeatherProvider>>,
    cache: Option<WeatherCache>,
    op_state: OpState,
}

impl WeatherModelRust {
    fn ensure_initialized(&mut self) {
        if self.provider.is_some() {
            return;
        }

        match crate::bridge::get_weather_services() {
            Some((provider, cache, _runtime)) => {
                self.provider = Some(provider);
                self.cache = Some(cache);
                tracing::info!("WeatherModel auto-initialized from global services");
            }
            None => {
                tracing::error!("Cannot auto-initialize WeatherModel - global services not ready");
            }
        }
    }

    fn store_weather_data(&mut self, data: &WeatherData) {
        self.weather_data = Some(data.clone());
    }

    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }
}

impl qobject::WeatherModel {
    /// Update all properties from weather data using Qt setters for proper notifications
    fn update_from_data(mut self: Pin<&mut Self>, data: &WeatherData) {
        // Current weather
        self.as_mut().set_temperature(data.current.temperature);
        self.as_mut().set_feels_like(data.current.feels_like);
        self.as_mut().set_humidity(data.current.humidity as i32);
        self.as_mut().set_wind_speed(data.current.wind_speed);
        self.as_mut().set_condition(QString::from(data.current.condition.description()));
        self.as_mut().set_condition_icon(QString::from(data.current.condition.icon_name()));

        let location_name =
            data.location.city_name.as_deref().map(QString::from).unwrap_or_else(|| {
                QString::from(format!(
                    "{:.2}, {:.2}",
                    data.location.latitude, data.location.longitude
                ))
            });
        self.as_mut().set_location_name(location_name);

        // Today's forecast (first day)
        if let Some(today) = data.forecast.first() {
            self.as_mut().set_today_high(today.high);
            self.as_mut().set_today_low(today.low);
            self.as_mut().set_precipitation_chance(today.precipitation_chance as i32);
            self.as_mut().set_sunrise(QString::from(today.sunrise.format("%H:%M").to_string()));
            self.as_mut().set_sunset(QString::from(today.sunset.format("%H:%M").to_string()));
        }

        // Store weather data for forecast methods
        self.as_mut().rust_mut().store_weather_data(data);
        self.as_mut().set_has_data(true);
    }

    /// Refresh weather data asynchronously (non-blocking)
    pub fn refresh(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("refresh: operation already in progress");
            return;
        }

        let provider = match &self.as_ref().rust().provider {
            Some(p) => p.clone(),
            None => {
                self.as_mut().set_error_message(QString::from("Weather service not initialized"));
                self.as_mut().error_occurred();
                return;
            }
        };

        // Check cache first - extract data to avoid borrow conflicts
        let cache_result: Option<(WeatherData, bool, bool)> =
            self.as_ref().rust().cache.as_ref().and_then(|cache| {
                if !cache.is_expired() {
                    cache.get().map(|data| (data.clone(), cache.is_stale(), cache.is_expired()))
                } else {
                    None
                }
            });

        if let Some((cached_data, is_stale, _)) = cache_result {
            tracing::info!("Using cached weather data");
            self.as_mut().update_from_data(&cached_data);
            self.as_mut().set_is_stale(is_stale);
            self.as_mut().weather_changed();

            // If not stale, we're done
            if !is_stale {
                return;
            }
            // Otherwise, continue to refresh in background
        }

        // Initialize channel if needed
        bridge::init_weather_service_channel();
        let tx = match bridge::get_weather_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Service channel not ready"));
                self.as_mut().error_occurred();
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::Fetching;

        // Spawn async operation (non-blocking)
        request_weather_fetch(&tx, provider);
    }

    /// Poll for async operation results. Call this from a QML Timer (e.g., every 100ms).
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_weather_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            WeatherServiceMessage::FetchDone(result) => {
                self.as_mut().set_loading(false);
                self.as_mut().rust_mut().op_state = OpState::Idle;

                match result {
                    Ok(data) => {
                        tracing::info!("Weather data fetched successfully");

                        // Update cache
                        if let Some(cache) = &mut self.as_mut().rust_mut().cache {
                            cache.update(data.clone());
                            if let Err(e) = cache.save() {
                                tracing::warn!("Failed to save weather cache: {}", e);
                            }
                        }

                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().update_from_data(&data);
                        self.as_mut().set_is_stale(false);
                        self.as_mut().weather_changed();
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch weather: {}", e);
                        self.as_mut()
                            .rust_mut()
                            .set_error(myme_core::AppError::from(e).user_message());
                        self.as_mut().error_occurred();

                        // If we have cached data, mark as stale but keep showing it
                        if self.as_ref().rust().has_data {
                            self.as_mut().set_is_stale(true);
                        }
                    }
                }
            }
        }
    }

    pub fn set_temperature_unit(mut self: Pin<&mut Self>, unit: &QString) {
        let unit_enum = match unit.to_string().to_lowercase().as_str() {
            "celsius" => TemperatureUnit::Celsius,
            "fahrenheit" => TemperatureUnit::Fahrenheit,
            _ => TemperatureUnit::Auto,
        };

        // Check if provider exists
        let has_provider = self.as_ref().rust().provider.is_some();

        if has_provider {
            // Create new provider with updated unit
            if let Ok(new_provider) = WeatherProvider::new(unit_enum) {
                self.as_mut().rust_mut().provider = Some(Arc::new(new_provider));
                // Refresh to get data in new unit
                self.refresh();
            }
        }
    }

    pub fn forecast_count(&self) -> i32 {
        self.rust().weather_data.as_ref().map(|d| d.forecast.len() as i32).unwrap_or(0)
    }

    pub fn get_forecast_date(&self, index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| QString::from(f.date.format("%Y-%m-%d").to_string()))
            .unwrap_or_default()
    }

    pub fn get_forecast_day(&self, index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| QString::from(f.date.format("%a").to_string()))
            .unwrap_or_default()
    }

    pub fn get_forecast_high(&self, index: i32) -> f64 {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| f.high)
            .unwrap_or(0.0)
    }

    pub fn get_forecast_low(&self, index: i32) -> f64 {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| f.low)
            .unwrap_or(0.0)
    }

    pub fn get_forecast_condition(&self, index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| QString::from(f.condition.description()))
            .unwrap_or_default()
    }

    pub fn get_forecast_icon(&self, index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| QString::from(f.condition.icon_name()))
            .unwrap_or_default()
    }

    pub fn get_forecast_precip(&self, index: i32) -> i32 {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| f.precipitation_chance as i32)
            .unwrap_or(0)
    }

    pub fn get_forecast_sunrise(&self, index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| QString::from(f.sunrise.format("%H:%M").to_string()))
            .unwrap_or_default()
    }

    pub fn get_forecast_sunset(&self, index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
            .map(|f| QString::from(f.sunset.format("%H:%M").to_string()))
            .unwrap_or_default()
    }

    pub fn hourly_count(&self, day_index: i32) -> i32 {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(day_index as usize))
            .map(|f| f.hourly.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_hourly_time(&self, day_index: i32, hour_index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(day_index as usize))
            .and_then(|f| f.hourly.get(hour_index as usize))
            .map(|h| QString::from(h.time.format("%H:%M").to_string()))
            .unwrap_or_default()
    }

    pub fn get_hourly_temp(&self, day_index: i32, hour_index: i32) -> f64 {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(day_index as usize))
            .and_then(|f| f.hourly.get(hour_index as usize))
            .map(|h| h.temperature)
            .unwrap_or(0.0)
    }

    pub fn get_hourly_icon(&self, day_index: i32, hour_index: i32) -> QString {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(day_index as usize))
            .and_then(|f| f.hourly.get(hour_index as usize))
            .map(|h| QString::from(h.condition.icon_name()))
            .unwrap_or_default()
    }

    pub fn get_hourly_precip(&self, day_index: i32, hour_index: i32) -> i32 {
        self.rust()
            .weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(day_index as usize))
            .and_then(|f| f.hourly.get(hour_index as usize))
            .map(|h| h.precipitation_chance as i32)
            .unwrap_or(0)
    }
}
