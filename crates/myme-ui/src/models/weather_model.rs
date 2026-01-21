use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_weather::{WeatherCache, WeatherData, WeatherProvider, TemperatureUnit};

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
    }
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
    runtime: Option<tokio::runtime::Handle>,
}

impl WeatherModelRust {
    fn ensure_initialized(&mut self) {
        if self.provider.is_some() && self.runtime.is_some() {
            return;
        }

        match crate::bridge::get_weather_services() {
            Some((provider, cache, runtime)) => {
                self.provider = Some(provider);
                self.cache = Some(cache);
                self.runtime = Some(runtime);
                tracing::info!("WeatherModel auto-initialized from global services");
            }
            None => {
                tracing::error!("Cannot auto-initialize WeatherModel - global services not ready");
            }
        }
    }

    fn update_from_data(&mut self, data: &WeatherData) {
        self.temperature = data.current.temperature;
        self.feels_like = data.current.feels_like;
        self.humidity = data.current.humidity as i32;
        self.wind_speed = data.current.wind_speed;
        self.condition = QString::from(data.current.condition.description());
        self.condition_icon = QString::from(data.current.condition.icon_name());
        self.location_name = data
            .location
            .city_name
            .as_deref()
            .map(QString::from)
            .unwrap_or_else(|| {
                QString::from(format!(
                    "{:.2}, {:.2}",
                    data.location.latitude, data.location.longitude
                ))
            });

        // Today's forecast (first day)
        if let Some(today) = data.forecast.first() {
            self.today_high = today.high;
            self.today_low = today.low;
            self.precipitation_chance = today.precipitation_chance as i32;
            self.sunrise = QString::from(today.sunrise.format("%H:%M").to_string());
            self.sunset = QString::from(today.sunset.format("%H:%M").to_string());
        }

        self.weather_data = Some(data.clone());
        self.has_data = true;
    }
}

impl qobject::WeatherModel {
    pub fn refresh(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        let provider = match &self.as_ref().rust().provider {
            Some(p) => p.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Weather service not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        // Check cache first
        if let Some(cache) = &self.as_ref().rust().cache {
            if !cache.is_expired() {
                if let Some(cached_data) = cache.get() {
                    tracing::info!("Using cached weather data");
                    self.as_mut().rust_mut().update_from_data(cached_data);
                    self.as_mut().set_is_stale(cache.is_stale());
                    self.weather_changed();

                    // If not stale, we're done
                    if !cache.is_stale() {
                        return;
                    }
                    // Otherwise, continue to refresh in background
                }
            }
        }

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        // Get location and fetch weather
        let result = runtime.block_on(async {
            let location = myme_weather::location::get_current_location().await?;
            tracing::info!(
                "Got location: {}, {}",
                location.latitude,
                location.longitude
            );
            provider.fetch(&location).await
        });

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

                self.as_mut().rust_mut().update_from_data(&data);
                self.as_mut().set_loading(false);
                self.as_mut().set_is_stale(false);
                self.weather_changed();
            }
            Err(e) => {
                tracing::error!("Failed to fetch weather: {}", e);
                self.as_mut()
                    .set_error_message(QString::from(format!("Weather error: {}", e)));
                self.as_mut().set_loading(false);

                // If we have cached data, mark as stale but keep showing it
                if self.as_ref().rust().has_data {
                    self.as_mut().set_is_stale(true);
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

        if let Some(provider) = &self.as_ref().rust().provider {
            // Create new provider with updated unit
            if let Ok(mut new_provider) = WeatherProvider::new(unit_enum) {
                new_provider.set_unit(unit_enum);
                self.as_mut().rust_mut().provider = Some(Arc::new(new_provider));
                // Refresh to get data in new unit
                self.refresh();
            }
        }
    }

    pub fn forecast_count(&self) -> i32 {
        self.rust()
            .weather_data
            .as_ref()
            .map(|d| d.forecast.len() as i32)
            .unwrap_or(0)
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
