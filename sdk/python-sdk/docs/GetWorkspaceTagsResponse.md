# GetWorkspaceTagsResponse

The response to a request to retrieve all workspace tags

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**workspace_tags** | [**List[FullWorkspaceTag]**](FullWorkspaceTag.md) |  | 

## Example

```python
from kraken_sdk.models.get_workspace_tags_response import GetWorkspaceTagsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetWorkspaceTagsResponse from a JSON string
get_workspace_tags_response_instance = GetWorkspaceTagsResponse.from_json(json)
# print the JSON string representation of the object
print GetWorkspaceTagsResponse.to_json()

# convert the object into a dict
get_workspace_tags_response_dict = get_workspace_tags_response_instance.to_dict()
# create an instance of GetWorkspaceTagsResponse from a dict
get_workspace_tags_response_form_dict = get_workspace_tags_response.from_dict(get_workspace_tags_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


