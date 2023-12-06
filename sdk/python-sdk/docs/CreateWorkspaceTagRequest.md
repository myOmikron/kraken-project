# CreateWorkspaceTagRequest

The request to create a workspace tag

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | Name of the tag | 
**color** | [**Color**](Color.md) |  | 

## Example

```python
from kraken_sdk.models.create_workspace_tag_request import CreateWorkspaceTagRequest

# TODO update the JSON string below
json = "{}"
# create an instance of CreateWorkspaceTagRequest from a JSON string
create_workspace_tag_request_instance = CreateWorkspaceTagRequest.from_json(json)
# print the JSON string representation of the object
print CreateWorkspaceTagRequest.to_json()

# convert the object into a dict
create_workspace_tag_request_dict = create_workspace_tag_request_instance.to_dict()
# create an instance of CreateWorkspaceTagRequest from a dict
create_workspace_tag_request_form_dict = create_workspace_tag_request.from_dict(create_workspace_tag_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


