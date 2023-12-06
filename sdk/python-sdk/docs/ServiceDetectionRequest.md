# ServiceDetectionRequest

The request to start a service detection

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**leech_uuid** | **str** | The leech to use  Leave empty to use a random leech | [optional] 
**address** | **str** | The ip address the service listens on | 
**port** | **int** | The port the service listens on | 
**timeout** | **int** | Time to wait for a response after sending the payload (or after establishing a connection, if not payload is to be sent)  The timeout is specified in milliseconds. | 
**workspace_uuid** | **str** | The workspace to execute the attack in | 

## Example

```python
from kraken_sdk.models.service_detection_request import ServiceDetectionRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ServiceDetectionRequest from a JSON string
service_detection_request_instance = ServiceDetectionRequest.from_json(json)
# print the JSON string representation of the object
print ServiceDetectionRequest.to_json()

# convert the object into a dict
service_detection_request_dict = service_detection_request_instance.to_dict()
# create an instance of ServiceDetectionRequest from a dict
service_detection_request_form_dict = service_detection_request.from_dict(service_detection_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


