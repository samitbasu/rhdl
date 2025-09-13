
# IBUFDS sysclk ##########################################################
set_property IOSTANDARD LVDS_25 [get_ports { sysclk_p }]
set_property PACKAGE_PIN K4 [get_ports { sysclk_p }]
set_property IOSTANDARD LVDS_25 [get_ports { sysclk_n }]
set_property PACKAGE_PIN J4 [get_ports { sysclk_n }]
create_clock -period 5 [get_ports sysclk_p]
