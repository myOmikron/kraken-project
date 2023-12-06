# SimpleTcpPortScanResult

A simple representation of a tcp port scan result

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uuid** | **str** | The primary key | 
**attack** | **str** | The attack which produced this result | 
**created_at** | **datetime** | The point in time, this result was produced | 
**address** | **str** | The ip address a port was found on | 
**port** | **int** | The found port | 

## Example

```python
from kraken_sdk.models.simple_tcp_port_scan_result import SimpleTcpPortScanResult

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleTcpPortScanResult from a JSON string
simple_tcp_port_scan_result_instance = SimpleTcpPortScanResult.from_json(json)
# print the JSON string representation of the object
print SimpleTcpPortScanResult.to_json()

# convert the object into a dict
simple_tcp_port_scan_result_dict = simple_tcp_port_scan_result_instance.to_dict()
# create an instance of SimpleTcpPortScanResult from a dict
simple_tcp_port_scan_result_form_dict = simple_tcp_port_scan_result.from_dict(simple_tcp_port_scan_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


