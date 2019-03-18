/*
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
*/

#if $$ L and 1 or 0 $$
kernel void zero_LL(global float *L) {
  const int x = get_global_id(0);
  const int y = get_global_id(1);
  const int z = get_global_id(2);

  $L[x, y, z] = 0.0f;
}
#endif

#if $$ I and O and 1 or 0 $$
kernel void post_LL(global float *I, global float *O) {
  const int x = get_global_id(0);
  const int y = get_global_id(1);

  float i = $I[x, y, 0];
  float o = $O[x, y, 0];

  $O[x, y, 1] = $I[x, y, 1]*o/i;
  $O[x, y, 2] = $I[x, y, 2]*o/i;
}
#endif

#if $$ I and D and S and H and R and O and 1 or 0 $$
kernel void transform(global float *I,
                     global float *D,
                     global float *S,
                     global float *H,
                     global float *R,
                     global float *O,
                     const float m) // midpoint
{
  const int x = get_global_id(0);
  const int y = get_global_id(1);
  const int z = get_global_id(2);

  float i = $I[x, y, z];
  float d = $D[x, y, z]+1.0f;
  float s = $S[x, y, z]+1.0f;
  float h = $H[x, y, z]+1.0f;
  float r = $R[x, y, z];

  float o = i - m;
  float f = -2*pown(fabs(o), 3) + 3*pown(fabs(o), 2);

  if      (o<-r)    o = o*h;
  else if (o>r)     o = o*s;
  //else if (o<0.0f)  o = (1.0f-f)*o*d + f*(o*h - d*r*0.5f); // optionally offset shadows and highlights by detail
  //else if (o>=0.0f) o = (1.0f-f)*o*d + f*(o*s + d*r*0.5f);
  else if (o<0.0f)  o = (1.0f-f)*o*d + f*(o*h);
  else if (o>=0.0f) o = (1.0f-f)*o*d + f*(o*s);

  $O[x, y, z] = o;
}
#endif

#if $$ G and T and O and 1 or 0 $$
kernel void apply_LL(global float *G,
                  global float *T,
                  global float *O,
                  const int l,   // current available level
                  const int lvl) // total number of levels
{
  const int x = get_global_id(0);
  const int y = get_global_id(1);
  const int z = get_global_id(2);

  float g = $G[x, y, z];
  float t = $T[x, y, z];
  float o = $O[x, y, z];

  float v = g*lvl;
  int vl = (int)(floor(v));
  int vh = vl + 1;
  float vf = vh - v;
  vh = min(vh, lvl);

  if (l==vl)
    o += vf*t;
  else if (l==vh)
    o += (1.0f-vf)*t;

  $O[x, y, z] = o;
}
#endif