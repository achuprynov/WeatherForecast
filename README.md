# WeatherForecast

Описание задачи:
RESTful веб-сервис возвращающий прогноз погоды в заданном городе :
- на заданный день (текущий или следующие)
- на ближайшую неделю (коллекция из 5 дней)

В качестве источника данных используются веб-сервисы
 - http://api.apixu.com/v1/forecast.json?key=<apikey>&q=Moscow,ru&days=5
 - https://query.yahooapis.com/v1/public/yql?q=select%20*%20from%20weather.forecast%20where%20woeid%20in%20(select%20woeid%20from%20geo.places(1)%20where%20text%3D%22Moscow%2C%20ru%22)&format=json&env=store%3A%2F%2Fdatatables.org%2Falltableswithkeys
 
Результат - среднее значение по данным из обоих.

Формат запроса:
http://localhost:3000/weather_forecast?city={city name},{country code}&day={Today|Tomorrow|5day}

Параметы запроса:
{city name},{country code} = название города и код страны в формате ISO 3166
{Today|Tomorrow|5day} = Today - сегодня. Tomorrow - завтра, 5day - на ближайшие 5 дней

Формат ответа:
{ 
	"city" : "<city>", 
	"temperature_unit" : "C",
	"forecast" : [{"date" : "<day>", "temp_min" : <temp_min>, "temp_max" : <temp_max>}]
}

В случае ошибки ответ будет иметь формат:
{
	"error" : "<описание ошибки>"
}

Пример запроса:
http://localhost:3000/weather_forecast?city=Moscow,ru&day=5day

Пример ответа:
{
	"city":"Moscow,ru",
	"forecast":[
		{"date":"2018-01-28","temp_max":-3,"temp_min":-6},
		{"date":"2018-01-29","temp_max":-2,"temp_min":-8},
		{"date":"2018-01-30","temp_max":1,"temp_min":-4},
		{"date":"2018-01-31","temp_max":0,"temp_min":-6},
		{"date":"2018-02-01","temp_max":-4,"temp_min":-11}
	],
	"temperature_unit":"C"
}
