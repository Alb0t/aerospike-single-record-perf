service {
    cluster-name mydc
    proto-fd-max 15000
}
logging {
    file /var/log/aerospike.log {
        context any info
    }
}
network {
    service {
        access-address 172.17.0.3
        address any
        alternate-access-address 127.0.0.1
        port 3100
    }
    heartbeat {
        interval 150
        mesh-seed-address-port 172.17.0.3 3002
        mode mesh
        port 3002
        timeout 10
    }
    fabric {
        port 3001
    }
}
namespace test {
    default-ttl 0
    index-stage-size 128M
    replication-factor 2
    sindex-stage-size 128M
    storage-engine memory {
        data-size 1G
    }
}