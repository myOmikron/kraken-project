# SimpleAggregationSource

Numbers how many attacks of a certain kind found an aggregated model

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bruteforce_subdomains** | **int** | Bruteforce subdomains via DNS requests | 
**tcp_port_scan** | **int** | Scan tcp ports | 
**query_certificate_transparency** | **int** | Query certificate transparency | 
**query_dehashed** | **int** | Query the dehashed API | 
**host_alive** | **int** | Check if a host is reachable via icmp | 
**service_detection** | **int** | Detect the service that is running on a port | 
**dns_resolution** | **int** | Resolve domain names | 
**manual** | **bool** | Manually inserted | 

## Example

```python
from kraken_sdk.models.simple_aggregation_source import SimpleAggregationSource

# TODO update the JSON string below
json = "{}"
# create an instance of SimpleAggregationSource from a JSON string
simple_aggregation_source_instance = SimpleAggregationSource.from_json(json)
# print the JSON string representation of the object
print SimpleAggregationSource.to_json()

# convert the object into a dict
simple_aggregation_source_dict = simple_aggregation_source_instance.to_dict()
# create an instance of SimpleAggregationSource from a dict
simple_aggregation_source_form_dict = simple_aggregation_source.from_dict(simple_aggregation_source_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


