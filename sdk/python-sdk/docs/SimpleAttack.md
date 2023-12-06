# SimpleAttack

A simple version of an attack

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The identifier of the attack | 
**workspace_uuid** | **str** | The workspace this attack is attached to | 
**attack_type** | [**AttackType**](AttackType.md) |  | 
**started_by** | [**SimpleUser**](SimpleUser.md) |  | 
**finished_at** | **datetime** | If this is None, the attack is still running | [optional] 
**error** | **str** | If this field is set, the attack has finished with an error | [optional] 
**created_at** | **datetime** | The point in time this attack was started | 

## Example

```python
from kraken_sdk.models.simple_attack import SimpleAttack

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleAttack from a JSON string
simple_attack_instance = SimpleAttack.from_json(json)
# print the JSON string representation of the object
print SimpleAttack.to_json()

# convert the object into a dict
simple_attack_dict = simple_attack_instance.to_dict()
# create an instance of SimpleAttack from a dict
simple_attack_form_dict = simple_attack.from_dict(simple_attack_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


