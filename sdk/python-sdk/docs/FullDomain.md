# FullDomain

A full representation of a domain in a workspace

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key of the domain | 
**domain** | **str** | The domain&#39;s name | 
**comment** | **str** | A comment | 
**workspace** | **str** | The workspace this domain is in | 
**tags** | [**List[SimpleTag]**](SimpleTag.md) | The list of tags this domain has attached to | 
**sources** | [**SimpleAggregationSource**](SimpleAggregationSource.md) |  | 
**created_at** | **datetime** | The point in time, the record was created | 

## Example

```python
from kraken_sdk.models.full_domain import FullDomain

# TODO update the JSON string below
json = "{}"
# create an instance of FullDomain from a JSON string
full_domain_instance = FullDomain.from_json(json)
# print the JSON string representation of the object
print FullDomain.to_json()

# convert the object into a dict
full_domain_dict = full_domain_instance.to_dict()
# create an instance of FullDomain from a dict
full_domain_form_dict = full_domain.from_dict(full_domain_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


