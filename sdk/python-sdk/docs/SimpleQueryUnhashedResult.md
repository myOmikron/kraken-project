# SimpleQueryUnhashedResult

A simple representation of a query unhashed result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key | 
**attack** | **str** | The attack which produced this result | 
**created_at** | **datetime** | The point in time, this result was produced | 
**dehashed_id** | **int** | ID of the entry | 
**email** | **str** | An email address | [optional] 
**username** | **str** | An username | [optional] 
**password** | **str** | A password | [optional] 
**hashed_password** | **str** | An hashed password | [optional] 
**ip_address** | **str** | An ip address | 
**name** | **str** | A name | [optional] 
**vin** | **str** | A vin | [optional] 
**address** | **str** | An address | [optional] 
**phone** | **str** | A phone number | [optional] 
**database_name** | **str** | A database name | [optional] 

## Example

```python
from kraken_sdk.models.simple_query_unhashed_result import SimpleQueryUnhashedResult

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleQueryUnhashedResult from a JSON string
simple_query_unhashed_result_instance = SimpleQueryUnhashedResult.from_json(json)
# print the JSON string representation of the object
print SimpleQueryUnhashedResult.to_json()

# convert the object into a dict
simple_query_unhashed_result_dict = simple_query_unhashed_result_instance.to_dict()
# create an instance of SimpleQueryUnhashedResult from a dict
simple_query_unhashed_result_form_dict = simple_query_unhashed_result.from_dict(simple_query_unhashed_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


