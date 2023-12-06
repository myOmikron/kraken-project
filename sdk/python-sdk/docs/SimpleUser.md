# SimpleUser

This struct holds the user information.  Note that `username` is unique, but as it is changeable, identify the user by its `uuid`

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** |  | 
**username** | **str** |  | 
**display_name** | **str** |  | 

## Example

```python
from kraken_sdk.models.simple_user import SimpleUser

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleUser from a JSON string
simple_user_instance = SimpleUser.from_json(json)
# print the JSON string representation of the object
print SimpleUser.to_json()

# convert the object into a dict
simple_user_dict = simple_user_instance.to_dict()
# create an instance of SimpleUser from a dict
simple_user_form_dict = simple_user.from_dict(simple_user_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


