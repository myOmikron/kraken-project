# FullService

A full representation of a service

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | Uuid of the service | 
**name** | **str** | The service&#39;s name | 
**version** | **str** | An optional version of the running service | [optional] 
**certainty** | [**ServiceCertainty**](ServiceCertainty.md) |  | 
**host** | [**SimpleHost**](SimpleHost.md) |  | 
**port** | [**SimplePort**](SimplePort.md) |  | [optional] 
**comment** | **str** | A comment to the service | 
**workspace** | **str** | The workspace this service is linked to | 
**tags** | [**List[SimpleTag]**](SimpleTag.md) | The tags this service is linked to | 
**sources** | [**SimpleAggregationSource**](SimpleAggregationSource.md) |  | 
**created_at** | **datetime** | The point in time, the record was created | 

## Example

```python
from kraken_sdk.models.full_service import FullService

# TODO update the JSON string below
json = "{}"
# create an instance of FullService from a JSON string
full_service_instance = FullService.from_json(json)
# print the JSON string representation of the object
print FullService.to_json()

# convert the object into a dict
full_service_dict = full_service_instance.to_dict()
# create an instance of FullService from a dict
full_service_form_dict = full_service.from_dict(full_service_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


