# UpdateHostRequest

The request to update a host

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**comment** | **str** |  | [optional] 
**global_tags** | **List[str]** |  | [optional] 
**workspace_tags** | **List[str]** |  | [optional] 

## Example

```python
from kraken_sdk.models.update_host_request import UpdateHostRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateHostRequest from a JSON string
update_host_request_instance = UpdateHostRequest.from_json(json)
# print the JSON string representation of the object
print UpdateHostRequest.to_json()

# convert the object into a dict
update_host_request_dict = update_host_request_instance.to_dict()
# create an instance of UpdateHostRequest from a dict
update_host_request_form_dict = update_host_request.from_dict(update_host_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


