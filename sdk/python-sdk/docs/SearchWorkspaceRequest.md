# SearchWorkspaceRequest

Request to search the workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**search_term** | **str** | the term to search for | 

## Example

```python
from kraken_sdk.models.search_workspace_request import SearchWorkspaceRequest

# TODO update the JSON string below
json = "{}"
# create an instance of SearchWorkspaceRequest from a JSON string
search_workspace_request_instance = SearchWorkspaceRequest.from_json(json)
# print the JSON string representation of the object
print SearchWorkspaceRequest.to_json()

# convert the object into a dict
search_workspace_request_dict = search_workspace_request_instance.to_dict()
# create an instance of SearchWorkspaceRequest from a dict
search_workspace_request_form_dict = search_workspace_request.from_dict(search_workspace_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


