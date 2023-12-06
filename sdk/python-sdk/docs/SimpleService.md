# SimpleService

A simple representation of a service

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**name** | **str** |  | 
**version** | **str** |  | [optional] 
**host** | **str** |  | 
**port** | **str** |  | [optional] 
**comment** | **str** |  | 
**workspace** | **str** |  | 
**created_at** | **datetime** | The point in time, the record was created | 

## Example

```python
from kraken_sdk.models.simple_service import SimpleService

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleService from a JSON string
simple_service_instance = SimpleService.from_json(json)
# print the JSON string representation of the object
print SimpleService.to_json()

# convert the object into a dict
simple_service_dict = simple_service_instance.to_dict()
# create an instance of SimpleService from a dict
simple_service_form_dict = simple_service.from_dict(simple_service_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


