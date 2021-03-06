--[[
  Copyright (C) 2011-2021 G. Bajlekov

    Ivy is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Ivy is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
]]

local proc = require "lib.opencl.process.ivy".new()

local source = [[
kernel greenRedMask(I, C, O)
  const x = get_global_id(0)
  const y = get_global_id(1)

  var i = I[x, y, 1]*0.5 + 0.5
	i = clamp(i, 0.0, 1.0)

  var lowIdx = clamp(floor(i*255), 0.0, 255.0)
	var highIdx = clamp(ceil(i*255), 0.0, 255.0)

	var lowVal = C[lowIdx]
	var highVal = C[highIdx]

	var factor = 0.0
  if lowIdx==highIdx then
    factor = 1.0
  else
    factor = (i*255.0-lowIdx)/(highIdx-lowIdx)
  end

	O[x, y, 0] = mix(lowVal, highVal, factor)
end
]]

local function execute()
	local I, C, O = proc:getAllBuffers(3)
	proc:executeKernel("greenRedMask", proc:size2D(O), {I, C, O})
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceString(source)
	return execute
end

return init
