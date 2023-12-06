# UpdateWorkspaceTag

The request to update a workspace tag

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** |  | [optional] 
**color** | [**Color**](Color.md) |  | [optional] 

## Example

```python
from kraken_sdk.models.update_workspace_tag import UpdateWorkspaceTag

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateWorkspaceTag from a JSON string
update_workspace_tag_instance = UpdateWorkspaceTag.from_json(json)
# print the JSON string representation of the object
print UpdateWorkspaceTag.to_json()

# convert the object into a dict
update_workspace_tag_dict = update_workspace_tag_instance.to_dict()
# create an instance of UpdateWorkspaceTag from a dict
update_workspace_tag_form_dict = update_workspace_tag.from_dict(update_workspace_tag_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


