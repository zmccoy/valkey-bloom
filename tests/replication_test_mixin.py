import pytest, os

class ReplicationTestMixin:
    """
    Shared replication test setup for the bloom and cms replication suites.

    Mix into a test class ahead of ReplicationTestCase, e.g.
        class TestBloomReplication(ReplicationTestMixin, ReplicationTestCase):

    Provides the autouse `setup_test` fixture (primary/replica server creation,
    either against an external server or a locally spawned one) and the
    `validate_cmd_stats` helper.
    """

    def module_load_args(self):
        """
        Extra args passed to the local server on top of the common ones.
        Override in a subclass to add module-specific config.
        """
        return {}

    @pytest.fixture(autouse=True)
    def setup_test(self, setup):
        use_external = os.environ.get("VALKEY_EXTERNAL_SERVER", "false").lower() == "true"

        if use_external:
            master_host = os.environ.get("VALKEY_HOST", "localhost")
            master_port = int(os.environ.get("VALKEY_PORT", "6379"))
            self.server, self.client = self.create_server(
                testdir=self.testdir,
                bind_ip=master_host,
                port=master_port,
                external_server=True
            )

            replica_host = os.environ.get("VALKEY_REPLICA_HOST", "localhost")
            replica_port = int(os.environ.get("VALKEY_REPLICA_PORT", "6380"))
            replica_server, replica_client = self.create_server(
                testdir=self.testdir,
                bind_ip=replica_host,
                port=replica_port,
                external_server=True
            )

            self.replicas = [replica_server]
            self.num_replicas = 1
        else:
            self.args = {"enable-debug-command": "yes", 'loadmodule': os.getenv('MODULE_PATH'), **self.module_load_args()}
            server_path = f"{os.path.dirname(os.path.realpath(__file__))}/build/binaries/{os.environ['SERVER_VERSION']}/valkey-server"
            self.server, self.client = self.create_server(testdir=self.testdir, server_path=server_path, args=self.args)

    def validate_cmd_stats(self, primary_cmd, replica_cmd, expected_primary_calls, expected_replica_calls):
        """
            Helper fn to validate cmd count on primary & replica.
        """
        primary_cmd_stats = self.client.info("Commandstats")['cmdstat_' + primary_cmd]
        assert primary_cmd_stats["calls"] == expected_primary_calls
        replica_cmd_stats = self.replicas[0].client.info("Commandstats")['cmdstat_' + replica_cmd]
        assert replica_cmd_stats["calls"] == expected_replica_calls
