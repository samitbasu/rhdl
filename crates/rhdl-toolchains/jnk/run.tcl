create_project demo jnk -part xc7a50tfgg484-1 -force
create_ip -name mig_7series -vendor xilinx.com -library ip -version 4.2 -module_name mig7
set_property CONFIG.BOARD_MIG_PARAM Custom [get_ips mig7]
set_property CONFIG.MIG_DONT_TOUCH_PARAM Custom [get_ips mig7]
set_property CONFIG.RESET_BOARD_INTERFACE Custom [get_ips mig7]
set_property CONFIG.XML_INPUT_FILE /home/samitbasu/Devel/rhdl/rhdl-bsp/jnk/mig_a.prj [get_ips mig7]
generate_target all [get_files mig7.xci]
