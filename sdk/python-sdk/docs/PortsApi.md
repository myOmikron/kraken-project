# kraken_sdk.PortsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_port**](PortsApi.md#create_port) | **POST** /api/v1/workspaces/{uuid}/ports | Manually add a port
[**get_all_ports**](PortsApi.md#get_all_ports) | **POST** /api/v1/workspaces/{uuid}/ports/all | List the ports of a workspace
[**get_port**](PortsApi.md#get_port) | **GET** /api/v1/workspaces/{w_uuid}/ports/{p_uuid} | Retrieve all information about a single port
[**update_port**](PortsApi.md#update_port) | **PUT** /api/v1/workspaces/{w_uuid}/ports/{p_uuid} | Update a port


# **create_port**
> UuidResponse create_port(uuid, create_port_request)

Manually add a port

Manually add a port

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.create_port_request import CreatePortRequest
from kraken_sdk.models.uuid_response import UuidResponse
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.PortsApi(api_client)
    uuid = 'uuid_example' # str | 
    create_port_request = kraken_sdk.CreatePortRequest() # CreatePortRequest | 

    try:
        # Manually add a port
        api_response = await api_instance.create_port(uuid, create_port_request)
        print("The response of PortsApi->create_port:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PortsApi->create_port: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **create_port_request** | [**CreatePortRequest**](CreatePortRequest.md)|  | 

### Return type

[**UuidResponse**](UuidResponse.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Port was created |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_all_ports**
> PortResultsPage get_all_ports(uuid, get_all_ports_query)

List the ports of a workspace

List the ports of a workspace

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.get_all_ports_query import GetAllPortsQuery
from kraken_sdk.models.port_results_page import PortResultsPage
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.PortsApi(api_client)
    uuid = 'uuid_example' # str | 
    get_all_ports_query = kraken_sdk.GetAllPortsQuery() # GetAllPortsQuery | 

    try:
        # List the ports of a workspace
        api_response = await api_instance.get_all_ports(uuid, get_all_ports_query)
        print("The response of PortsApi->get_all_ports:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PortsApi->get_all_ports: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **str**|  | 
 **get_all_ports_query** | [**GetAllPortsQuery**](GetAllPortsQuery.md)|  | 

### Return type

[**PortResultsPage**](PortResultsPage.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Retrieve all ports of a workspace |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_port**
> FullPort get_port(w_uuid, p_uuid)

Retrieve all information about a single port

Retrieve all information about a single port

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.full_port import FullPort
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.PortsApi(api_client)
    w_uuid = 'w_uuid_example' # str | The workspace's uuid
    p_uuid = 'p_uuid_example' # str | The port's uuid

    try:
        # Retrieve all information about a single port
        api_response = await api_instance.get_port(w_uuid, p_uuid)
        print("The response of PortsApi->get_port:\n")
        pprint(api_response)
    except Exception as e:
        print("Exception when calling PortsApi->get_port: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**| The workspace&#39;s uuid | 
 **p_uuid** | **str**| The port&#39;s uuid | 

### Return type

[**FullPort**](FullPort.md)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Retrieved the selected port |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **update_port**
> update_port(w_uuid, p_uuid, update_port_request)

Update a port

Update a port  You must include at least on parameter

### Example

* Api Key Authentication (api_key):
```python
import time
import os
import kraken_sdk
from kraken_sdk.models.update_port_request import UpdatePortRequest
from kraken_sdk.rest import ApiException
from pprint import pprint

# Defining the host is optional and defaults to http://localhost
# See configuration.py for a list of all supported configuration parameters.
configuration = kraken_sdk.Configuration(
    host = "http://localhost"
)

# The client must configure the authentication and authorization parameters
# in accordance with the API server security policy.
# Examples for each auth method are provided below, use the example that
# satisfies your auth use case.

# Configure API key authorization: api_key
configuration.api_key['api_key'] = os.environ["API_KEY"]

# Uncomment below to setup prefix (e.g. Bearer) for API key, if needed
# configuration.api_key_prefix['api_key'] = 'Bearer'

# Enter a context with an instance of the API client
async with kraken_sdk.ApiClient(configuration) as api_client:
    # Create an instance of the API class
    api_instance = kraken_sdk.PortsApi(api_client)
    w_uuid = 'w_uuid_example' # str | The workspace's uuid
    p_uuid = 'p_uuid_example' # str | The port's uuid
    update_port_request = kraken_sdk.UpdatePortRequest() # UpdatePortRequest | 

    try:
        # Update a port
        await api_instance.update_port(w_uuid, p_uuid, update_port_request)
    except Exception as e:
        print("Exception when calling PortsApi->update_port: %s\n" % e)
```



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **w_uuid** | **str**| The workspace&#39;s uuid | 
 **p_uuid** | **str**| The port&#39;s uuid | 
 **update_port_request** | [**UpdatePortRequest**](UpdatePortRequest.md)|  | 

### Return type

void (empty response body)

### Authorization

[api_key](../README.md#api_key)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Port was updated |  -  |
**400** | Client error |  -  |
**500** | Server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

