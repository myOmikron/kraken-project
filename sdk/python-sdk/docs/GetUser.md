# GetUser

A single user representation

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**username** | **str** |  | 
**display_name** | **str** |  | 
**permission** | [**UserPermission**](UserPermission.md) |  | 
**created_at** | **datetime** |  | 
**last_login** | **datetime** |  | [optional] 

## Example

```python
from kraken_sdk.models.get_user import GetUser

# TODO update the JSON string below
json = "{}"
# create an instance of GetUser from a JSON string
get_user_instance = GetUser.from_json(json)
# print the JSON string representation of the object
print GetUser.to_json()

# convert the object into a dict
get_user_dict = get_user_instance.to_dict()
# create an instance of GetUser from a dict
get_user_form_dict = get_user.from_dict(get_user_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


