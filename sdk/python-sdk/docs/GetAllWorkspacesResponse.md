# GetAllWorkspacesResponse

The response to retrieve all workspaces

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**workspaces** | [**List[SimpleWorkspace]**](SimpleWorkspace.md) |  | 

## Example

```python
from kraken_sdk.models.get_all_workspaces_response import GetAllWorkspacesResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetAllWorkspacesResponse from a JSON string
get_all_workspaces_response_instance = GetAllWorkspacesResponse.from_json(json)
# print the JSON string representation of the object
print GetAllWorkspacesResponse.to_json()

# convert the object into a dict
get_all_workspaces_response_dict = get_all_workspaces_response_instance.to_dict()
# create an instance of GetAllWorkspacesResponse from a dict
get_all_workspaces_response_form_dict = get_all_workspaces_response.from_dict(get_all_workspaces_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


