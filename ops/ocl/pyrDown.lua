--[[
  Copyright (C) 2011-2018 G. Bajlekov

    ImageFloat is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    ImageFloat is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
]]

local proc = require "lib.opencl.process".new()

local function execute()
	proc:getAllBuffers("I", "L", "G")
	proc:executeKernel("pyrDown", proc:size3D("G"), {"I", "G"})
	proc:executeKernel("pyrUpL", proc:size3D("G"))
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceFile("pyr.cl")
	return execute
end

return init
