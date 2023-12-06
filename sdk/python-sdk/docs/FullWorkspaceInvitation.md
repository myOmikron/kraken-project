# FullWorkspaceInvitation

The full representation of an invitation to a workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The uuid of the invitation | 
**workspace** | [**SimpleWorkspace**](SimpleWorkspace.md) |  | 
**var_from** | [**SimpleUser**](SimpleUser.md) |  | 
**target** | [**SimpleUser**](SimpleUser.md) |  | 

## Example

```python
from kraken_sdk.models.full_workspace_invitation import FullWorkspaceInvitation

# TODO update the JSON string below
json = "{}"
# create an instance of FullWorkspaceInvitation from a JSON string
full_workspace_invitation_instance = FullWorkspaceInvitation.from_json(json)
# print the JSON string representation of the object
print FullWorkspaceInvitation.to_json()

# convert the object into a dict
full_workspace_invitation_dict = full_workspace_invitation_instance.to_dict()
# create an instance of FullWorkspaceInvitation from a dict
full_workspace_invitation_form_dict = full_workspace_invitation.from_dict(full_workspace_invitation_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


