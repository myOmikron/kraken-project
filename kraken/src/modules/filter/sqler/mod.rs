mod joins;
mod value_sqler;

use std::fmt;
use std::fmt::Write;

use rorm::crud::selector::Selector;
use rorm::db::sql::value::Value;
use rorm::prelude::*;

use crate::models::Domain;
use crate::models::DomainDomainRelation;
use crate::models::DomainHostRelation;
use crate::models::Host;
use crate::models::HttpService;
use crate::models::Port;
use crate::models::Service;
use crate::modules::filter::sqler::joins::from_http_service_join_domain;
use crate::modules::filter::sqler::joins::from_http_service_join_host;
use crate::modules::filter::sqler::joins::from_http_service_join_port;
use crate::modules::filter::sqler::joins::from_port_join_host;
use crate::modules::filter::sqler::joins::from_service_join_host;
use crate::modules::filter::sqler::joins::from_service_join_port;
use crate::modules::filter::sqler::joins::JoinTags;
use crate::modules::filter::sqler::value_sqler::Column;
use crate::modules::filter::sqler::value_sqler::ValueSqler;
use crate::modules::filter::And;
use crate::modules::filter::DomainAST;
use crate::modules::filter::GlobalAST;
use crate::modules::filter::HostAST;
use crate::modules::filter::HttpServiceAST;
use crate::modules::filter::Not;
use crate::modules::filter::Or;
use crate::modules::filter::PortAST;
use crate::modules::filter::ServiceAST;
use crate::modules::raw_query::RawQueryBuilder;

impl DomainAST {
    /// Apply the domain specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::domain());
        }

        let DomainAST {
            tags,
            created_at,
            domains,
            source_of,
            source_of_tags,
            source_of_created_at,
            target_of,
            target_of_tags,
            target_of_created_at,
            ips,
            ips_created_at,
            ips_tags,
            ips_os,
        } = self;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Domain::F.created_at).range());
        add_ast_field(sql, domains, Column::rorm(Domain::F.domain).eq());

        if source_of.is_some() || source_of_tags.is_some() || source_of_created_at.is_some() {
            sql.append_condition(Column::rorm(Domain::F.uuid).in_subquery(
                DomainDomainRelation::F.source,
                |sql| {
                    sql.append_join(|sql: &mut String, _: &mut Vec<Value>| {
                        write!(
                            sql,
                            r#" JOIN "domain" ON "domain"."uuid" = "domaindomainrelation"."destination""#
                        )
                    });
                    if source_of_tags.is_some() {
                        sql.append_join(JoinTags::domain());
                    }

                    add_ast_field(sql, source_of, Column::rorm(Domain::F.domain).eq());
                    add_ast_field(sql, source_of_tags, Column::tags().contains());
                    add_ast_field(sql, source_of_created_at, Column::rorm(Domain::F.created_at).range());
                },
            ));
        }

        if target_of.is_some() || target_of_tags.is_some() || target_of_created_at.is_some() {
            sql.append_condition(Column::rorm(Domain::F.uuid).in_subquery(
                DomainDomainRelation::F.destination,
                |sql| {
                    sql.append_join(|sql: &mut String, _: &mut Vec<Value>| {
                        write!(
                            sql,
                            r#" JOIN "domain" ON "domain"."uuid" = "domaindomainrelation"."source""#
                        )
                    });
                    if target_of_tags.is_some() {
                        sql.append_join(JoinTags::domain());
                    }

                    add_ast_field(sql, target_of, Column::rorm(Domain::F.domain).eq());
                    add_ast_field(sql, target_of_tags, Column::tags().contains());
                    add_ast_field(
                        sql,
                        target_of_created_at,
                        Column::rorm(Domain::F.created_at).range(),
                    );
                },
            ));
        }

        // Sub query the hosts
        if ips.is_some() || ips_tags.is_some() || ips_created_at.is_some() || ips_os.is_some() {
            sql.append_condition(Column::rorm(Domain::F.uuid).in_subquery(
                DomainHostRelation::F.domain,
                |sql| {
                    sql.append_join(|sql: &mut String, _: &mut Vec<Value>| {
                        write!(
                            sql,
                            r#" JOIN "host" ON "host"."uuid" = "domainhostrelation"."host""#
                        )
                    });

                    if ips_tags.is_some() {
                        sql.append_join(JoinTags::host());
                    }

                    add_ast_field(sql, ips, Column::rorm(Host::F.ip_addr).subnet());
                    add_ast_field(sql, ips_tags, Column::tags().contains());
                    add_ast_field(
                        sql,
                        ips_created_at,
                        Column::rorm(Host::F.created_at).range(),
                    );
                    add_ast_field(sql, ips_os, Column::rorm(Host::F.os_type).eq());
                },
            ));
        }

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Domain::F.created_at).range());
    }
}
impl HostAST {
    /// Apply the host specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::host());
        }

        let HostAST {
            tags,
            created_at,
            ips,
            os,
            ports,
            ports_created_at,
            ports_protocols,
            ports_tags,
            services,
            services_ports,
            services_protocols,
            services_tags,
            services_created_at,
            services_transports,
            domains,
            domains_tags,
            domains_created_at,
        } = self;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Host::F.created_at).range());
        add_ast_field(sql, ips, Column::rorm(Host::F.ip_addr).subnet());
        add_ast_field(sql, os, Column::rorm(Host::F.os_type).eq());

        // Sub query the ports
        if ports.is_some()
            || ports_created_at.is_some()
            || ports_protocols.is_some()
            || ports_tags.is_some()
        {
            sql.append_condition(Column::rorm(Host::F.uuid).in_subquery(Port::F.host, |sql| {
                if ports_tags.is_some() {
                    sql.append_join(JoinTags::port());
                }

                add_ast_field(sql, ports, Column::rorm(Port::F.port).maybe_range());
                add_ast_field(
                    sql,
                    ports_created_at,
                    Column::rorm(Port::F.created_at).range(),
                );
                add_ast_field(sql, ports_protocols, Column::rorm(Port::F.protocol).eq());
                add_ast_field(sql, ports_tags, Column::tags().contains());
            }));
        }

        // Sub query the services
        if services.is_some()
            || services_ports.is_some()
            || services_protocols.is_some()
            || services_tags.is_some()
            || services_created_at.is_some()
            || services_transports.is_some()
        {
            sql.append_condition(
                Column::rorm(Host::F.uuid).in_subquery(Service::F.host, |sql| {
                    if services_tags.is_some() {
                        sql.append_join(JoinTags::service());
                    }
                    if services_ports.is_some() || services_protocols.is_some() {
                        sql.append_join(from_service_join_port());
                    }

                    add_ast_field(sql, services, Column::rorm(Service::F.name).eq());
                    add_ast_field(
                        sql,
                        services_ports,
                        Column::rorm(Port::F.port).nullable_maybe_range(),
                    );
                    add_ast_field(
                        sql,
                        services_protocols,
                        Column::rorm(Port::F.protocol).nullable_eq(),
                    );
                    add_ast_field(sql, services_tags, Column::tags().contains());
                    add_ast_field(
                        sql,
                        services_created_at,
                        Column::rorm(Service::F.created_at).range(),
                    );
                    add_ast_field(
                        sql,
                        services_transports,
                        Column::rorm(Service::F.protocols).bitset(),
                    );
                }),
            );
        }

        // Sub query the domains
        if domains.is_some() || domains_tags.is_some() || domains_created_at.is_some() {
            sql.append_condition(Column::rorm(Host::F.uuid).in_subquery(
                DomainHostRelation::F.host,
                |sql| {
                    sql.append_join(|sql: &mut String, _: &mut Vec<Value>| {
                        write!(
                            sql,
                            r#" JOIN "domain" ON "domain"."uuid" = "domainhostrelation"."domain""#
                        )
                    });

                    if domains_tags.is_some() {
                        sql.append_join(JoinTags::domain());
                    }

                    add_ast_field(sql, domains, Column::rorm(Domain::F.domain).eq());
                    add_ast_field(sql, domains_tags, Column::tags().contains());
                    add_ast_field(
                        sql,
                        domains_created_at,
                        Column::rorm(Domain::F.created_at).range(),
                    );
                },
            ));
        }

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Host::F.created_at).range());
    }
}
impl PortAST {
    /// Apply the port specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::port());
        }

        if self.ips_created_at.is_some() || self.ips_tags.is_some() || self.ips_os.is_some() {
            sql.append_join(from_port_join_host());
        }

        if self.ips_tags.is_some() {
            sql.append_join(JoinTags::host().alias("host_tags"));
        }

        let PortAST {
            tags,
            created_at,
            ports,
            ips,
            ips_created_at,
            ips_tags,
            ips_os,
            protocols,
            services,
            services_tags,
            services_created_at,
            services_transports,
        } = self;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Port::F.created_at).range());
        add_ast_field(sql, ports, Column::rorm(Port::F.port).maybe_range());
        add_ast_field(sql, ips, Column::rorm(Port::F.host.ip_addr).subnet());
        add_ast_field(sql, protocols, Column::rorm(Port::F.protocol).eq());
        add_ast_field(
            sql,
            ips_created_at,
            Column::rorm(Host::F.created_at).range(),
        );
        add_ast_field(sql, ips_os, Column::rorm(Host::F.os_type).eq());
        add_ast_field(sql, ips_tags, Column::new("host_tags", "tags").contains());

        // Sub query the services
        if services.is_some()
            || services_tags.is_some()
            || services_created_at.is_some()
            || services_transports.is_some()
        {
            sql.append_condition(
                Column::rorm(Port::F.uuid).in_subquery(Service::F.port, |sql| {
                    if services_tags.is_some() {
                        sql.append_join(JoinTags::service());
                    }

                    add_ast_field(sql, services, Column::rorm(Service::F.name).eq());
                    add_ast_field(sql, services_tags, Column::tags().contains());
                    add_ast_field(
                        sql,
                        services_created_at,
                        Column::rorm(Service::F.created_at).range(),
                    );
                    add_ast_field(
                        sql,
                        services_transports,
                        Column::rorm(Service::F.protocols).bitset(),
                    );
                }),
            );
        }

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Port::F.created_at).range());
    }
}
impl ServiceAST {
    /// Apply the service specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::service());
        }
        if self.ports.is_some()
            || self.ports_created_at.is_some()
            || self.protocols.is_some()
            || self.ports_tags.is_some()
        {
            sql.append_join(from_service_join_port());
        }
        if self.ports_tags.is_some() {
            sql.append_join(JoinTags::port().alias("port_tags")); // TODO: does this work since port might be null?
        }
        if self.ips_created_at.is_some() || self.ips_tags.is_some() || self.ips_os.is_some() {
            sql.append_join(from_service_join_host());
        }
        if self.ips_tags.is_some() {
            sql.append_join(JoinTags::host().alias("host_tags"));
        }

        let ServiceAST {
            tags,
            created_at,
            ips,
            ips_created_at,
            ips_tags,
            ips_os,
            ports_tags,
            ports_created_at,
            protocols,
            services,
            ports,
            transport,
        } = self;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Service::F.created_at).range());
        add_ast_field(sql, ips, Column::rorm(Service::F.host.ip_addr).subnet());
        add_ast_field(sql, services, Column::rorm(Service::F.name).eq());
        add_ast_field(sql, transport, Column::rorm(Service::F.protocols).bitset());
        add_ast_field(
            sql,
            ports,
            Column::rorm(Port::F.port).nullable_maybe_range(),
        );
        add_ast_field(
            sql,
            ports_created_at,
            Column::rorm(Port::F.created_at).nullable_range(),
        );
        add_ast_field(sql, ports_tags, Column::new("port_tags", "tags").contains());
        add_ast_field(sql, protocols, Column::rorm(Port::F.protocol).nullable_eq());
        add_ast_field(
            sql,
            ips_created_at,
            Column::rorm(Host::F.created_at).range(),
        );
        add_ast_field(sql, ips_os, Column::rorm(Host::F.os_type).eq());
        add_ast_field(sql, ips_tags, Column::new("host_tags", "tags").contains());

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Service::F.created_at).range());
    }
}
impl HttpServiceAST {
    /// Apply the http service specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::http_service());
        }
        if self.ports.is_some()
            || self.ports_created_at.is_some()
            || self.ports_tags.is_some()
            || self.ports_protocols.is_some()
        {
            sql.append_join(from_http_service_join_port());
        }
        if self.ports_tags.is_some() {
            sql.append_join(JoinTags::port().alias("port_tags"));
        }
        if self.ips_created_at.is_some() || self.ips_tags.is_some() || self.ips_os.is_some() {
            sql.append_join(from_http_service_join_host());
        }
        if self.ips_tags.is_some() {
            sql.append_join(JoinTags::host().alias("host_tags"));
        }
        if self.domains.is_some()
            || self.domains_created_at.is_some()
            || self.domains_tags.is_some()
        {
            sql.append_join(from_http_service_join_domain());
        }
        if self.domains_tags.is_some() {
            sql.append_join(JoinTags::domain().alias("domain_tags")); // TODO: does this work since domain might be null?
        }

        let HttpServiceAST {
            tags,
            created_at,
            ips,
            ips_created_at,
            ips_tags,
            ips_os,
            ports,
            ports_tags,
            ports_created_at,
            ports_protocols,
            domains,
            domains_tags,
            domains_created_at,
            http_services,
            base_paths,
            tls,
            sni,
        } = self;

        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(
            sql,
            created_at,
            Column::rorm(HttpService::F.created_at).range(),
        );

        add_ast_field(sql, ips, Column::rorm(HttpService::F.host.ip_addr).subnet());
        add_ast_field(sql, http_services, Column::rorm(HttpService::F.name).eq());
        add_ast_field(sql, base_paths, Column::rorm(HttpService::F.base_path).eq());
        add_ast_field(sql, tls, Column::rorm(HttpService::F.tls).eq());
        add_ast_field(sql, sni, Column::rorm(HttpService::F.sni_required).eq());
        add_ast_field(
            sql,
            ports,
            Column::rorm(Port::F.port).nullable_maybe_range(),
        );
        add_ast_field(
            sql,
            ports_created_at,
            Column::rorm(Port::F.created_at).nullable_range(),
        );
        add_ast_field(sql, ports_protocols, Column::rorm(Port::F.protocol).eq());
        add_ast_field(sql, ports_tags, Column::new("port_tags", "tags").contains());
        add_ast_field(
            sql,
            ips_created_at,
            Column::rorm(Host::F.created_at).range(),
        );
        add_ast_field(sql, ips_os, Column::rorm(Host::F.os_type).eq());
        add_ast_field(sql, ips_tags, Column::new("host_tags", "tags").contains());

        add_ast_field(
            sql,
            domains_created_at,
            Column::rorm(Domain::F.created_at).range(),
        );
        add_ast_field(sql, domains, Column::rorm(Domain::F.domain).eq());
        add_ast_field(
            sql,
            domains_tags,
            Column::new("domain_tags", "tags").contains(),
        );

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(
            sql,
            created_at,
            Column::rorm(HttpService::F.created_at).range(),
        );
    }
}

/// Helper function to be called from `...AST::apply_to_query`
///
/// It checks a field to be `Some` and appends it to the query builder
pub fn add_ast_field<'a, T>(
    query_builder: &mut RawQueryBuilder<'a, impl Selector>,
    field: &'a Option<Or<T>>,
    sql_value: impl ValueSqler<T>,
) {
    if let Some(field) = field {
        query_builder.append_condition(|sql, values| sql_or(field, sql, values, sql_value))
    }
}

/// Write an [`Or`] expression to sql using a [`ValueSqler`] to write the leaves
pub fn sql_or<'a, T>(
    or: &'a Or<T>,
    sql: &mut String,
    values: &mut Vec<Value<'a>>,
    sql_value: impl ValueSqler<T>,
) -> fmt::Result {
    match or.0.as_slice() {
        [] => write!(sql, "true"),
        [and] => sql_and(and, sql, values, &sql_value),
        [first, rest @ ..] => {
            write!(sql, "(")?;
            sql_and(first, sql, values, &sql_value)?;
            for and in rest {
                write!(sql, " OR ")?;
                sql_and(and, sql, values, &sql_value)?;
            }
            write!(sql, ")")?;
            Ok(())
        }
    }
}

/// Write an [`And`] expression to sql using a [`ValueSqler`] to write the leaves
pub fn sql_and<'a, T>(
    and: &'a And<T>,
    sql: &mut String,
    values: &mut Vec<Value<'a>>,
    sql_value: &impl ValueSqler<T>,
) -> fmt::Result {
    match and.0.as_slice() {
        [] => write!(sql, "false"),
        [not] => sql_not(not, sql, values, sql_value),
        [first, rest @ ..] => {
            write!(sql, "(")?;
            sql_not(first, sql, values, sql_value)?;
            for not in rest {
                write!(sql, " AND ")?;
                sql_not(not, sql, values, sql_value)?;
            }
            write!(sql, ")")?;
            Ok(())
        }
    }
}

/// Write a [`Not`] to sql expression using a [`ValueSqler`] to write the potentially negated value
pub fn sql_not<'a, T>(
    not: &'a Not<T>,
    sql: &mut String,
    values: &mut Vec<Value<'a>>,
    sql_value: &impl ValueSqler<T>,
) -> fmt::Result {
    if not.is_negated {
        write!(sql, "(NOT ")?;
        sql_value.sql_value(&not.value, sql, values)?;
        write!(sql, ")")?;
        Ok(())
    } else {
        sql_value.sql_value(&not.value, sql, values)
    }
}
