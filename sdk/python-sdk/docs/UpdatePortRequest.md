# UpdatePortRequest

The request to update a port

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**comment** | **str** |  | [optional] 
**global_tags** | **List[str]** |  | [optional] 
**workspace_tags** | **List[str]** |  | [optional] 

## Example

```python
from kraken_sdk.models.update_port_request import UpdatePortRequest

# TODO update the JSON string below
json = "{}"
# create an instance of UpdatePortRequest from a JSON string
update_port_request_instance = UpdatePortRequest.from_json(json)
# print the JSON string representation of the object
print UpdatePortRequest.to_json()

# convert the object into a dict
update_port_request_dict = update_port_request_instance.to_dict()
# create an instance of UpdatePortRequest from a dict
update_port_request_form_dict = update_port_request.from_dict(update_port_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


