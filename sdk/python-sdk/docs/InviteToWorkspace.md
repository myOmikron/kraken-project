# InviteToWorkspace

The request to invite a user to the workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user** | **str** | The user to invite | 

## Example

```python
from kraken_sdk.models.invite_to_workspace import InviteToWorkspace

# TODO update the JSON string below
json = "{}"
# create an instance of InviteToWorkspace from a JSON string
invite_to_workspace_instance = InviteToWorkspace.from_json(json)
# print the JSON string representation of the object
print InviteToWorkspace.to_json()

# convert the object into a dict
invite_to_workspace_dict = invite_to_workspace_instance.to_dict()
# create an instance of InviteToWorkspace from a dict
invite_to_workspace_form_dict = invite_to_workspace.from_dict(invite_to_workspace_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


