# FullServiceDetectionResult

A simple representation of a service detection result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key | 
**attack** | **str** | The attack which produced this result | 
**created_at** | **datetime** | The point in time, this result was produced | 
**certainty** | **str** | The certainty a service is detected | 
**service_names** | **List[str]** | The found names of the service | 
**host** | **str** | The ip address a port was found on | 
**port** | **int** | Port number | 

## Example

```python
from kraken_sdk.models.full_service_detection_result import FullServiceDetectionResult

# TODO update the JSON string below
json = "{}"
# create an instance of FullServiceDetectionResult from a JSON string
full_service_detection_result_instance = FullServiceDetectionResult.from_json(json)
# print the JSON string representation of the object
print FullServiceDetectionResult.to_json()

# convert the object into a dict
full_service_detection_result_dict = full_service_detection_result_instance.to_dict()
# create an instance of FullServiceDetectionResult from a dict
full_service_detection_result_form_dict = full_service_detection_result.from_dict(full_service_detection_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


