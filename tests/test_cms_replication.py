import os
from valkey import ResponseError
from valkeytestframework.valkey_test_case import ReplicationTestCase
from valkeytestframework.conftest import resource_port_tracker
from replication_test_mixin import ValkeyBloomReplicationTestCaseBase

class TestCMSReplication(ValkeyBloomReplicationTestCaseBase, ReplicationTestCase):

    def reset_state(self):
        self.client.execute_command('FLUSHALL')
        self.waitForReplicaToSyncUp(self.replicas[0])
        self.client.execute_command('CONFIG RESETSTAT')
        self.replicas[0].client.execute_command('CONFIG RESETSTAT')

    def compare_info(self):
        primary_info_call = self.client.execute_command('CMS.INFO key')
        replica_info_call = self.replicas[0].client.execute_command('CMS.INFO key')
        assert primary_info_call == replica_info_call
        

    def test_replication_behavior(self):
        use_external = os.environ.get("VALKEY_EXTERNAL_SERVER", "false").lower() == "true"
        if use_external:
            self.wait_for_primary_link_up_all_replicas()
        else:
            self.setup_replication(num_replicas=1)

        # Test replication for write commands. Shape: (prefix, setup, item_add, expectedCalls)
        cms_write_cmds = [
            ('CMS.INITBYDIM', 'CMS.INITBYDIM key 10 5', 'CMS.INCRBY key item1 1', 1),
            ('CMS.INITBYPROB', 'CMS.INITBYPROB key 0.001 0.01', 'CMS.INCRBY key item1 1', 1),
        ]
        
        for test_case in cms_write_cmds:
            prefix = test_case[0]
            create_cmd = test_case[1]
            # New cms object being created is replicated.
            self.client.execute_command(create_cmd)
            assert self.client.execute_command('EXISTS key') == 1
            self.waitForReplicaToSyncUp(self.replicas[0])
            assert self.replicas[0].client.execute_command('EXISTS key') == 1
            self.validate_cmd_stats(prefix, prefix, 1, 1)

            # New item added to an existing cms is replicated.
            item_add_cmd = test_case[2]
            expected_calls = test_case[3]

            self.validate_cmd_stats(prefix, prefix, expected_calls, expected_calls)

            self.client.execute_command(item_add_cmd)
            self.waitForReplicaToSyncUp(self.replicas[0])

            self.validate_cmd_stats('CMS.INCRBY', 'CMS.INCRBY', 1, 1)
            
            assert self.client.execute_command('CMS.QUERY key item1') == [1]
            assert self.replicas[0].client.execute_command('CMS.QUERY key item1') == [1]
           
            self.compare_info()
            
            # cmd debug digest
            # TO BE IMPLEMENTED ONCE DIGEST IS IMPLEMENTED

            self.reset_state()
        

        # Merge commands setup and test
        cms_merge_setup = [
            'CMS.INITBYDIM key 10 5', # Needs to exist for Merge, empty
            'CMS.INITBYDIM key2 10 5',
            'CMS.INCRBY key2 item1 1',
            'CMS.INITBYDIM key3 10 5',
            'CMS.INCRBY key3 item1 1',
        ]

        for setup in cms_merge_setup:
            self.client.execute_command(setup)

        self.client.execute_command('CMS.MERGE key 2 key2 key3')

        # These assertions only prove that the INITBYDIM created the key. 
        assert self.client.execute_command('EXISTS key') == 1
        self.waitForReplicaToSyncUp(self.replicas[0])
        assert self.replicas[0].client.execute_command('EXISTS key') == 1

        self.validate_cmd_stats('CMS.MERGE', 'CMS.MERGE', 1, 1)
        
        self.compare_info()
        
        self.reset_state()
        
        # Re-setup for read commands
        self.client.execute_command('CMS.INITBYDIM key 10 5')
        self.client.execute_command('CMS.INCRBY key item1 1')
        self.waitForReplicaToSyncUp(self.replicas[0])

        # Read commands executed on the primary will not be replicated.
        read_commands = [
            ('CMS.QUERY', 'CMS.QUERY key item1', 1),
            ('CMS.INFO', 'CMS.INFO key', 1),
            ('CMS.INFO', 'CMS.INFO key WIDTH', 2),
            ('CMS.INFO', 'CMS.INFO key DEPTH', 3),
            ('CMS.INFO', 'CMS.INFO key COUNT', 4),
        ]
        for test_case in read_commands:
            prefix = test_case[0]
            cmd = test_case[1]
            expected_primary_calls = test_case[2]
            self.client.execute_command(cmd)
            primary_cmd_stats = self.client.info("Commandstats")['cmdstat_' + prefix]
            assert primary_cmd_stats["calls"] == expected_primary_calls
            assert ('cmdstat_' + prefix) not in self.replicas[0].client.info("Commandstats")

        # Reset the stats for the invalid write commands testing below
        self.reset_state()

        # Write commands with errors are not replicated. Shape: (prefix, setup_commands, command, number_of_calls, failed_calls)
        invalid_write_cmds = [
            ('CMS.INITBYDIM', [], 'CMS.INITBYDIM key 10 5 5', 1, 1), # Bad parsing too many arguments
            ('CMS.INITBYDIM', ['CMS.INITBYPROB key 0.001 0.01'], 'CMS.INITBYDIM key 10 5', 1, 1), # Fail to create as the key already exists.
            ('CMS.INITBYPROB', [], 'CMS.INITBYPROB key 0 0.1', 1, 1), # Bad parsing 0 is not a valid value
            ('CMS.INCRBY', [], 'CMS.INCRBY key item1 1 1', 1, 1), # Bad parsing too many arguments
            ('CMS.INCRBY', [], 'CMS.INCRBY noExist item1 1', 1, 1), # Key does not exist
            ('CMS.INCRBY', ['CMS.INITBYDIM key 10 5'], 'CMS.INCRBY key item1 a', 1, 1), # Key exists, Bad value of increment
            ('CMS.MERGE', [], 'CMS.MERGE key', 1, 1), # Bad parsing too few arguments
            ('CMS.MERGE', [], 'CMS.MERGE key dest1 dest2', 1, 1), # Key does not exist
            ('CMS.MERGE', ['CMS.INITBYDIM key 10 5', 'CMS.INITBYDIM dest2 10 5'], 'CMS.MERGE key dest1 dest2', 1, 1), # Dest1 does not exist
            ('CMS.MERGE', ['CMS.INITBYDIM key 10 5', 'CMS.INITBYDIM dest1 10 5'], 'CMS.MERGE key dest1 dest2', 1, 1), # Dest2 does not exist
            ('CMS.MERGE', ['CMS.INITBYDIM key 10 5', 'CMS.INITBYDIM dest1 10 5', 'CMS.INITBYDIM dest2 10 5'], 'CMS.MERGE key dest1 dest2 WEIGHTS 3', 1, 1), # Source keys not equal to weights
            ('CMS.MERGE', ['CMS.INITBYDIM key 10 5', 'CMS.INITBYDIM dest1 10 5', 'CMS.INITBYDIM dest2 10 5'], 'CMS.MERGE key dest1 dest2 WEIGU 3 3', 1, 1), # WEIGHTS keyword is wrong
        ]
        for test_case in invalid_write_cmds:
            prefix = test_case[0]
            setup_cmds = test_case[1]
            cmd = test_case[2]
            expected_calls = test_case[3]
            failed_calls = test_case[4]

            # Earlier cases may leave `key` behind; remove it so each setup starts clean.
            # DEL does not touch the CMS command counts asserted below.
            self.client.execute_command('DEL key')
            for setup_command in setup_cmds:
                self.client.execute_command(setup_command)

            self.waitForReplicaToSyncUp(self.replicas[0])

            try:
                self.client.execute_command(cmd)
                assert False
            except ResponseError as e:
                pass
            primary_cmd_stats = self.client.info("Commandstats")['cmdstat_' + prefix]
            assert primary_cmd_stats["calls"] == expected_calls
            assert primary_cmd_stats["failed_calls"] == failed_calls
            assert ('cmdstat_' + prefix) not in self.replicas[0].client.info("Commandstats")

            self.reset_state()


