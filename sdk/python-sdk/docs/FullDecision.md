# FullDecision

A user's remembered oauth decision

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key | 
**app** | **str** | The application the decision was made for | 
**workspace** | [**SimpleWorkspace**](SimpleWorkspace.md) |  | 
**action** | **str** | Action what to do with new oauth requests | 

## Example

```python
from kraken_sdk.models.full_decision import FullDecision

# TODO update the JSON string below
json = "{}"
# create an instance of FullDecision from a JSON string
full_decision_instance = FullDecision.from_json(json)
# print the JSON string representation of the object
print FullDecision.to_json()

# convert the object into a dict
full_decision_dict = full_decision_instance.to_dict()
# create an instance of FullDecision from a dict
full_decision_form_dict = full_decision.from_dict(full_decision_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


