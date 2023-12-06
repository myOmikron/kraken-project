# UpdateServiceRequest

The request to update a service

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**comment** | **str** |  | [optional] 
**global_tags** | **List[str]** |  | [optional] 
**workspace_tags** | **List[str]** |  | [optional] 

## Example

```python
from kraken_sdk.models.update_service_request import UpdateServiceRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateServiceRequest from a JSON string
update_service_request_instance = UpdateServiceRequest.from_json(json)
# print the JSON string representation of the object
print UpdateServiceRequest.to_json()

# convert the object into a dict
update_service_request_dict = update_service_request_instance.to_dict()
# create an instance of UpdateServiceRequest from a dict
update_service_request_form_dict = update_service_request.from_dict(update_service_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


