--[[
  Copyright (C) 2011-2019 G. Bajlekov

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

local ffi = require "ffi"
local proc = require "lib.opencl.process".new()
local data = require "data"

local source = [[
#include "cs.cl"

// domain transform copy
kernel void derivative(global float *J, global float *dHdx, global float *dVdy, global float *S, global float *R)
{
  const int x = get_global_id(0);
  const int y = get_global_id(1);

	float3 jo = $J[x, y];
	float3 jx = $J[x+1, y];
	float3 jy = $J[x, y+1];

	float s = S[0];
	float r = R[0];
	float sr = s/fmax(r, 0.0001f);

	float3 h3 = fabs(jx-jo);
	float h = 1.0f + sr*(h3.x + h3.y + h3.z);

	float3 v3 = fabs(jy-jo);
	float v = 1.0f + sr*(v3.x + v3.y + v3.z);

	if (x+1<$dHdx.x$) $dHdx[x+1, y] = h;
	if (y+1<$dVdy.y$) $dVdy[x, y+1] = v;
}

kernel void horizontal(global float *dHdx, global float *O, global float *S, float h) {
	//const int x = get_global_id(0);
	const int y = get_global_id(1);

	for (int x = 1; x<$O.x$; x++) {
		float3 io = $O[x, y];
		float3 ix = $O[x-1, y];
		float a = exp( -sqrt(2.0f) / ($S[x, y]*h));
		float v = pow(a, $dHdx[x, y]);
		$O[x, y] = io + v * (ix - io);
	}

	for (int x = $O.x$-2; x>=0; x--) {
		float3 io = $O[x, y];
		float3 ix = $O[x+1, y];
		float a = exp( -sqrt(2.0f) / ($S[x+1, y]*h));
		float v = pow(a, $dHdx[x+1, y]);
		$O[x, y] = io + v * (ix - io);
	}
}

kernel void vertical(global float *dVdy, global float *O, global float *S, float h) {
	const int x = get_global_id(0);
	//const int y = get_global_id(1);

	for (int y = 1; y<$O.y$; y++) {
		float3 io = $O[x, y];
		float3 iy = $O[x, y-1];
		float a = exp( -sqrt(2.0f) / ($S[x, y]*h));
		float v = pow(a, $dVdy[x, y]);
		$O[x, y] = io + v * (iy - io);
	}

	for (int y = $O.y$-2; y>=0; y--) {
		float3 io = $O[x, y];
		float3 iy = $O[x, y+1];
		float a = exp( -sqrt(2.0f) / ($S[x, y+1]*h));
		float v = pow(a, $dVdy[x, y+1]);
		$O[x, y] = io + v * (iy - io);
	}
}




kernel void convert(
	global float *I,
	global float *M,
	global float *W,
	global float *P,
	global float *flags,
	global float *C
) {
	const int x = get_global_id(0);
	const int y = get_global_id(1);

	float3 i = $I[x, y];

	bool c = i.x>0.95f || i.y>0.95f || i.z>0.95f;
	$C[x, y] = c ? 1.0f : 0.0f;

	if (flags[3]>0.5f)
		i = i * $P[0, 0];

	if (c && flags[5]>0.5f)
		i = (float3)(LRGBtoY(i));

	if (flags[4]>0.5f)
		i = i * $W[0, 0];

	if (flags[3]>0.5f) {
		if (c && flags[5]<0.5f)
			i = clamp(i, 0.0f, 1.0f);

		float3 o = i;
		o.x = i.x*$M[0, 0, 0] + i.y*$M[0, 1, 0] + i.z*$M[0, 2, 0];
		o.y = i.x*$M[1, 0, 0] + i.y*$M[1, 1, 0] + i.z*$M[1, 2, 0];
		o.z = i.x*$M[2, 0, 0] + i.y*$M[2, 1, 0] + i.z*$M[2, 2, 0];

		if (c && flags[5]>0.5f)
			o = (float3)(LRGBtoY(o));

		$I[x, y] = o;
	} else {
		$I[x, y] = i;
	}
}

kernel void expand(global float *I, global float *C, global float *J, global float *O) {
	const int x = get_global_id(0);
	const int y = get_global_id(1);

	bool e = false;
	bool f = false;
	bool c = $C[x, y] > 0.5f;

	for (int i = -2; i<=2; i++)
		for (int j = -2; j<=2; j++)
			if ($C[x+i, y+j]>0.5f) f = true;

	for (int i = -4; i<=4; i++)
		for (int j = -4; j<=4; j++)
			if ($C[x+i, y+j]>0.5f) e = true;

	float3 i = $I[x, y];
	bool l = max(max(i.x, i.y), i.z) > 0.75f;

	$J[x, y] = e ? i : (float3)(0.0f);
	$O[x, y] = (e && !f && l) ? i : (float3)(0.0f);
}

kernel void merge(global float *I, global float *C, global float *O) {
	const int x = get_global_id(0);
	const int y = get_global_id(1);

	bool c = $C[x, y]>0.5f;

	if (c) {
		float3 i = $I[x, y];
		float3 o = $O[x, y];

		i = LRGBtoXYZ(i);
		o = LRGBtoXYZ(o);
		o = o * i.y/o.y;

		$I[x, y] = XYZtoLRGB(o);
	}
}

]]

local function execute()
	proc:getAllBuffers("I", "M", "W", "P", "flags")

	local x, y, z = proc.buffers.I:shape()
	proc.buffers.C = data:new(x, y, 1) -- clipping mask
	proc.buffers.J = data:new(x, y, z) -- guide

	proc.buffers.S = data:new(1, 1, 1) -- DT filter param
	proc.buffers.R = data:new(1, 1, 1) -- DT filter param
	proc.buffers.S:set(0, 0, 0, 50)
	proc.buffers.R:set(0, 0, 0, 0.5)
	proc.buffers.S:toDevice()
	proc.buffers.R:toDevice()

	proc.buffers.dHdx = data:new(x, y, 1)
	proc.buffers.dVdy = data:new(x, y, 1)
	proc.buffers.O = data:new(x, y, z) -- reference in, reconstructed colors out

	proc:executeKernel("convert", proc:size2D("I"), {"I", "M", "W", "P", "flags", "C"})

	if proc.buffers.flags:get(0, 0, 5) > 0.5 then
		proc:executeKernel("expand", proc:size2D("I"), {"I", "C", "J", "O"})

		-- DT dx, dy generate dHdx, dVdy from G
		proc:executeKernel("derivative", proc:size2D("I"), {"J", "dHdx", "dVdy", "S", "R"})

		-- DT iterate V, H over R with G as guide
		local N = 5 -- number of iterations
		local h = ffi.new("float[1]")
		for i = 0, N-1 do
			h[0] = math.sqrt(3) * 2^(N - (i+1)) / math.sqrt(4^N - 1)
			proc:executeKernel("vertical", {x, 1}, {"dVdy", "O", "S", h})
			proc:executeKernel("horizontal", {1, y}, {"dHdx", "O", "S", h})
		end

		-- merge colors from R in I according to C
		proc:executeKernel("merge", proc:size2D("I"), {"I", "C", "O"})
	end
end

local function init(d, c, q)
	proc:init(d, c, q)
	proc:loadSourceString(source)
	return execute
end

return init
