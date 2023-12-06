# CreateServiceRequest

The request to manually add a service

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | The service&#39;s name | 
**certainty** | [**ManualServiceCertainty**](ManualServiceCertainty.md) |  | 
**host** | **str** | The ip address the service runs on | 
**port** | **int** | An optional port the service runs on | [optional] 

## Example

```python
from kraken_sdk.models.create_service_request import CreateServiceRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateServiceRequest from a JSON string
create_service_request_instance = CreateServiceRequest.from_json(json)
# print the JSON string representation of the object
print CreateServiceRequest.to_json()

# convert the object into a dict
create_service_request_dict = create_service_request_instance.to_dict()
# create an instance of CreateServiceRequest from a dict
create_service_request_form_dict = create_service_request.from_dict(create_service_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


