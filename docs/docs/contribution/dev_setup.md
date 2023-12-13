# Development Environment

## Setting up a development environment

The easiest way to get kraken running locally is using Vagrant and the provided Vagrantfile, which starts and uses virtual machines that take care of most things automatically, including compilation, through Ansible.

### `vagrant/vars.yml`

Copy the `example.vars.yml` to `vars.yml` in the vagrant folder and set test values for the tokens, names and passwords. A sample vars.yml might start like this:

```yml
--8<-- "example.vars.yml:sample"
  ...
```

The server will only listen to requests on the `origin_uri` URL. For this example, you can add `127.0.0.1   kraken.localhost` to your `/etc/hosts` file.

The `kraken_secret_key` and `generated_leech_conf` blocks can first be left as-is and need to be filled with generated information after the first startup.

### Creating VMs

Before the first startup, if you don't have a high-spec machine with 8 or more cores, make sure to reduce the VM CPUs inside the root Vagrantfile in the checked out kraken repository. Using more VM CPUs than what are available on the host may cause hangups and slowdowns. If you change these values later, you can restart the VM with the config using `vagrant reload`

Now, initialize the VMs and perform a first compilation using

```
$ vagrant up
```

If you access http://kraken.localhost:8081 now, you should see a broken black page saying "Loading..." as well as showing errors.

### Setup admin user and kraken key

Now open a shell in the VM using

```
$ vagrant ssh
```

and inside this shell create an admin user by running
```
$ sudo kraken create-admin-user
Enter a username:
Enter a display name:
Enter password:
[2023-12-13 11:12:17 | DEBUG | rorm_db::database] SQL: SELECT "user"."uuid" AS user__uuid FROM "user" WHERE ("user".username = $1) LIMIT 1;
[2023-12-13 11:12:17 | DEBUG | rorm_db::database] SQL: INSERT INTO "user" ("uuid", "username", "display_name", "permission") VALUES ($1, $2, $3, 'Admin');
[2023-12-13 11:12:17 | DEBUG | rorm_db::database] SQL: INSERT INTO "localuser" ("uuid", "user", "password_hash") VALUES ($1, $2, $3);

Created user admin
```

Following this run keygen:

```
$ sudo kraken keygen
uRi7GQJkXSSM9oeYvOunX5X0izMSLOq5CItHC...
```

Copy the key that is output from this command and put it into `kraken_secret_key` inside your `vars.yml` file.

Now exit the shell with Ctrl-D and since we edited vars.yml you need to re-provision the VM:

```
$ vagrant provision kraken
```

Now reload the Kraken webpage (http://kraken.localhost:8081) and login using the credentials that you have entered previously.

## First-time configuration

Once logged in as admin, go to the "Kraken Network" page and click "Add leech"

Pick any name, for example "Default leech" and as address use

```
https://10.13.37.11:31337
```

When added to the list, click on "Gen tls config"

The TLS config will be copied to clipboard (KrakenSni, KrakenCa, etc.). Paste this configuration inside your `vars.yml` as value for the `generated_leech_conf` field. (note that you need to indent the string to avoid YAML syntax errors)

Now since vars.yml has been modified, you need to re-provision the leech VM again:

```
$ vagrant provision leech
```

The leech should be available now and attacks in workspaces can now be done.

## Internal files

The following paths are used for storage and configuration:
```
kraken:
	/etc/kraken/config.toml
	/var/lib/kraken

leech:
	/etc/leech/config.toml
```
